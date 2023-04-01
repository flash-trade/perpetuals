//! RemoveCollateral instruction handler

use {
    crate::{
        error::PerpetualsError,
        math,
        state::{
            custody::Custody, oracle::OraclePrice, perpetuals::Perpetuals, pool::Pool,
            position::Position,
        },
    },
    anchor_lang::prelude::*,
    anchor_spl::token::{Token, TokenAccount},
    solana_program::program_error::ProgramError,
};

#[derive(Accounts)]
#[instruction(params: RemoveCollateralParams)]
pub struct RemoveCollateral<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        mut,
        constraint = receiving_account.mint == collateral_custody.mint,
        has_one = owner
    )]
    pub receiving_account: Box<Account<'info, TokenAccount>>,

    /// CHECK: empty PDA, authority for token accounts
    #[account(
        seeds = [b"transfer_authority"],
        bump = perpetuals.transfer_authority_bump
    )]
    pub transfer_authority: AccountInfo<'info>,

    #[account(
        seeds = [b"perpetuals"],
        bump = perpetuals.perpetuals_bump
    )]
    pub perpetuals: Box<Account<'info, Perpetuals>>,

    #[account(
        mut,
        seeds = [b"pool",
                 pool.name.as_bytes()],
        bump = pool.bump
    )]
    pub pool: Box<Account<'info, Pool>>,

    #[account(
        mut,
        has_one = owner,
        seeds = [b"position",
                 owner.key().as_ref(),
                 pool.key().as_ref(),
                 custody.key().as_ref(),
                 &[position.side as u8]],
        bump = position.bump
    )]
    pub position: Box<Account<'info, Position>>,

    #[account(
        mut,
        seeds = [b"custody",
                 pool.key().as_ref(),
                 custody.mint.as_ref()],
        bump = custody.bump
    )]
    pub custody: Box<Account<'info, Custody>>,

    /// CHECK: oracle account for the collateral token
    #[account(
        constraint = custody_oracle_account.key() == custody.oracle.oracle_account
    )]
    pub custody_oracle_account: AccountInfo<'info>,


    #[account(
        mut,
        seeds = [b"custody",
                 pool.key().as_ref(),
                 collateral_custody.mint.as_ref()],
        bump = collateral_custody.bump
    )]
    pub collateral_custody: Box<Account<'info, Custody>>,

    /// CHECK: oracle account for the collateral token
    #[account(
        constraint = collateral_custody_oracle_account.key() == collateral_custody.oracle.oracle_account
    )]
    pub collateral_custody_oracle_account: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [b"custody_token_account",
                 pool.key().as_ref(),
                 collateral_custody.mint.as_ref()],
        bump = collateral_custody.token_account_bump
    )]
    pub collateral_custody_token_account: Box<Account<'info, TokenAccount>>,

    token_program: Program<'info, Token>,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct RemoveCollateralParams {
    collateral_usd: u64,
}

pub fn remove_collateral(
    ctx: Context<RemoveCollateral>,
    params: &RemoveCollateralParams,
) -> Result<()> {
    // check permissions
    msg!("Check permissions");
    let perpetuals = ctx.accounts.perpetuals.as_mut();
    let custody = ctx.accounts.custody.as_mut();
    let collateral_custody = ctx.accounts.collateral_custody.as_mut();
    let position = ctx.accounts.position.as_mut();
    let pool = ctx.accounts.pool.as_mut();
    let collateral_token_id = pool.get_token_id(&collateral_custody.key())?;
    require!(
        perpetuals.permissions.allow_collateral_withdrawal
            && custody.permissions.allow_collateral_withdrawal,
        PerpetualsError::InstructionNotAllowed
    );
    if position.collateral_custody == collateral_custody.key() {
        return Err(ProgramError::InvalidAccountData.into());
    }

    // validate inputs
    msg!("Validate inputs");
    if params.collateral_usd == 0 || params.collateral_usd >= position.collateral_usd {
        return Err(ProgramError::InvalidArgument.into());
    }

    // compute position price
    let curtime = perpetuals.get_time()?;

    let token_ema_price = OraclePrice::new_from_oracle(
        custody.oracle.oracle_type,
        &ctx.accounts.custody_oracle_account.to_account_info(),
        custody.oracle.max_price_error,
        custody.oracle.max_price_age_sec,
        curtime,
        custody.pricing.use_ema,
    )?;

    let collateral_token_price = OraclePrice::new_from_oracle(
        collateral_custody.oracle.oracle_type,
        &ctx.accounts.collateral_custody_oracle_account.to_account_info(),
        collateral_custody.oracle.max_price_error,
        collateral_custody.oracle.max_price_age_sec,
        curtime,
        false,
    )?;

    let collateral_token_ema_price = OraclePrice::new_from_oracle(
        collateral_custody.oracle.oracle_type,
        &ctx.accounts.collateral_custody_oracle_account.to_account_info(),
        collateral_custody.oracle.max_price_error,
        collateral_custody.oracle.max_price_age_sec,
        curtime,
        collateral_custody.pricing.use_ema,
    )?;

    let collateral_max_price = if collateral_token_price > collateral_token_ema_price {
        collateral_token_price
    } else {
        collateral_token_ema_price
    };

    // compute fee
    let collateral = 
        collateral_max_price.get_token_amount(params.collateral_usd, collateral_custody.decimals)?;
    let fee_amount =
        pool.get_remove_liquidity_fee(collateral_token_id, collateral, collateral_custody, &collateral_token_ema_price)?;
    msg!("Collected fee: {}", fee_amount);

    // compute amount to transfer
    if collateral > position.collateral_amount {
        return Err(ProgramError::InsufficientFunds.into());
    }
    let transfer_amount = math::checked_sub(collateral, fee_amount)?;
    msg!("Amount out: {}", transfer_amount);

    // update existing position
    msg!("Update existing position");
    position.update_time = perpetuals.get_time()?;
    position.collateral_usd = math::checked_sub(position.collateral_usd, params.collateral_usd)?;
    position.collateral_amount = math::checked_sub(position.collateral_amount, collateral)?;

    // check position risk
    msg!("Check position risks");
    require!(
        pool.check_leverage(position, &token_ema_price, custody, curtime, true)?,
        PerpetualsError::MaxLeverage
    );

    // transfer tokens
    msg!("Transfer tokens");
    perpetuals.transfer_tokens(
        ctx.accounts.collateral_custody_token_account.to_account_info(),
        ctx.accounts.receiving_account.to_account_info(),
        ctx.accounts.transfer_authority.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        transfer_amount,
    )?;

    // update custody stats
    msg!("Update custody stats");
    collateral_custody.collected_fees.open_position_usd = collateral_custody
        .collected_fees
        .open_position_usd
        .wrapping_add(collateral_token_ema_price.get_asset_amount_usd(fee_amount, custody.decimals)?);

    collateral_custody.assets.collateral = math::checked_sub(collateral_custody.assets.collateral, collateral)?;

    let protocol_fee = Pool::get_fee_amount(custody.fees.protocol_share, fee_amount)?;
    collateral_custody.assets.protocol_fees = math::checked_add(collateral_custody.assets.protocol_fees, protocol_fee)?;

    collateral_custody.remove_collateral(position.side, params.collateral_usd)?;

    Ok(())
}
