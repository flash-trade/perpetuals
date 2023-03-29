//! OpenPosition instruction handler

use {
    crate::{
        error::PerpetualsError,
        math,
        state::{
            custody::Custody,
            oracle::OraclePrice,
            perpetuals::Perpetuals,
            pool::Pool,
            position::{Position, Side},
        },
    },
    anchor_lang::prelude::*,
    anchor_spl::token::{Token, TokenAccount},
    solana_program::program_error::ProgramError,
};

#[derive(Accounts)]
#[instruction(params: OpenPositionParams)]
pub struct OpenPosition<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        mut,
        constraint = funding_account.mint == collateral_custody.mint,
        has_one = owner
    )]
    pub funding_account: Box<Account<'info, TokenAccount>>,

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
        init,
        payer = owner,
        space = Position::LEN,
        seeds = [b"position",
                 owner.key().as_ref(),
                 pool.key().as_ref(),
                 custody.key().as_ref(),
                 &[params.side as u8]],
        bump
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

    system_program: Program<'info, System>,
    token_program: Program<'info, Token>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy)]
// price and size are in custody whereas collateral is in collateral_custody
pub struct OpenPositionParams {
    pub price: u64,
    pub collateral: u64,
    pub size: u64,
    pub side: Side,
}

pub fn open_position(ctx: Context<OpenPosition>, params: &OpenPositionParams) -> Result<()> {
    // check permissions
    msg!("Check permissions");
    let perpetuals = ctx.accounts.perpetuals.as_mut();
    let custody = ctx.accounts.custody.as_mut();
    let mut collateral_custody = ctx.accounts.collateral_custody.as_mut();

    require!(
        perpetuals.permissions.allow_open_position
            && custody.permissions.allow_open_position
            && !custody.is_stable,
        PerpetualsError::InstructionNotAllowed
    );

    // validate inputs
    msg!("Validate inputs");
    if params.price == 0 || params.collateral == 0 || params.size == 0 || params.side == Side::None
    {
        return Err(ProgramError::InvalidArgument.into());
    }
    let position = ctx.accounts.position.as_mut();
    let pool = ctx.accounts.pool.as_mut();

    // compute position price
    let curtime = perpetuals.get_time()?;

    let token_price = OraclePrice::new_from_oracle(
        custody.oracle.oracle_type,
        &ctx.accounts.custody_oracle_account.to_account_info(),
        custody.oracle.max_price_error,
        custody.oracle.max_price_age_sec,
        curtime,
        false,
    )?;

    let token_ema_price = OraclePrice::new_from_oracle(
        custody.oracle.oracle_type,
        &ctx.accounts.custody_oracle_account.to_account_info(),
        custody.oracle.max_price_error,
        custody.oracle.max_price_age_sec,
        curtime,
        custody.pricing.use_ema,
    )?;

    let custody_min_price = if token_price < token_ema_price {
        token_price
    } else {
        token_ema_price
    };

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
        &ctx.accounts.custody_oracle_account.to_account_info(),
        collateral_custody.oracle.max_price_error,
        collateral_custody.oracle.max_price_age_sec,
        curtime,
        collateral_custody.pricing.use_ema,
    )?;

    let collateral_min_price = if collateral_token_price < collateral_token_ema_price {
        collateral_token_price
    } else {
        collateral_token_ema_price
    };

    let position_price =
        pool.get_entry_price(&token_price, &token_ema_price, params.side, custody)?;
    msg!("Entry price: {}", position_price);

    //locked_amount is in collateral_custody
    let locked_amount = if params.side == Side::Long {
        require_gte!(
            params.price,
            position_price,
            PerpetualsError::MaxPriceSlippage
        );

        if custody.key() != collateral_custody.key() {
            return Err(ProgramError::InvalidArgument.into());
        }
        math::checked_div(
            math::checked_mul(params.size as u128, custody.pricing.max_payoff_mult as u128)?,
            Perpetuals::BPS_POWER,
        )?
    } else {
        if collateral_custody.is_stable {
            return Err(ProgramError::InvalidArgument.into())
        }

        require_gte!(
            position_price,
            params.price,
            PerpetualsError::MaxPriceSlippage
        );

        let locked_usd = math::checked_div(
            math::checked_mul(
                math::checked_mul(params.size as u128, custody_min_price.price as u128)?,
                custody.pricing.max_payoff_mult as u128)?,
            Perpetuals::BPS_POWER,
        )?;

        collateral_min_price.get_token_amount(
            locked_usd.try_into().unwrap(), collateral_custody.decimals
        )?.into()
    };
 
    // compute fee
    let mut fee_amount = pool.get_entry_fee(
        params.size,
        locked_amount.try_into().unwrap(), 
        &token_price, 
        &collateral_min_price,
        custody,
        collateral_custody
    )?;

    msg!("Collected fee: {}", fee_amount);

    // compute amount to transfer
    let transfer_amount = math::checked_add(params.collateral, fee_amount)?;
    msg!("Amount in: {}", transfer_amount);

    // init new position
    msg!("Initialize new position");
    let size_usd = custody_min_price.get_asset_amount_usd(params.size, custody.decimals)?;
    let collateral_usd = collateral_min_price.get_asset_amount_usd(params.collateral, collateral_custody.decimals)?;
   
    position.owner = ctx.accounts.owner.key();
    position.pool = pool.key();
    position.custody = custody.key();
    position.open_time = curtime;
    position.update_time = 0;
    position.side = params.side;
    position.price = position_price;
    position.size_usd = size_usd;
    position.collateral_usd = collateral_usd;
    position.unrealized_profit_usd = 0;
    position.unrealized_loss_usd = 0;
    position.cumulative_interest_snapshot = custody.get_cumulative_interest(curtime)?;
    position.locked_amount = math::checked_as_u64(locked_amount)?;
    position.collateral_amount = params.collateral; 
    position.bump = *ctx
        .bumps
        .get("position")
        .ok_or(ProgramError::InvalidSeeds)?;

    // check position risk
    msg!("Check position risks");
    require!(
        position.locked_amount > 0,
        PerpetualsError::InsufficientAmountReturned
    );
    require!(
        pool.check_leverage(position, &token_ema_price, custody, curtime, true)?,
        PerpetualsError::MaxLeverage
    );

    // lock funds for potential profit payoff
    collateral_custody.lock_funds(position.locked_amount)?;

    // transfer tokens
    msg!("Transfer tokens");
    perpetuals.transfer_tokens_from_user(
        ctx.accounts.funding_account.to_account_info(),
        ctx.accounts.collateral_custody_token_account.to_account_info(),
        ctx.accounts.owner.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        transfer_amount,
    )?;

    // update custody stats
    msg!("Update custody stats");
    collateral_custody.collected_fees.open_position_usd = collateral_custody
        .collected_fees
        .open_position_usd
        .wrapping_add(collateral_token_ema_price.get_asset_amount_usd(fee_amount, collateral_custody.decimals)?);

    custody.volume_stats.open_position_usd = custody
        .volume_stats
        .open_position_usd
        .wrapping_add(size_usd);

    collateral_custody.assets.collateral = math::checked_add(collateral_custody.assets.collateral, params.collateral)?;

    let protocol_fee = Pool::get_fee_amount(custody.fees.protocol_share, fee_amount)?;
    collateral_custody.assets.protocol_fees = math::checked_add(collateral_custody.assets.protocol_fees, protocol_fee)?;

    if params.side == Side::Long {
        custody.trade_stats.oi_long_usd =
            math::checked_add(custody.trade_stats.oi_long_usd, size_usd)?;
    } else {
        custody.trade_stats.oi_short_usd =
            math::checked_add(custody.trade_stats.oi_short_usd, size_usd)?;
    }

    //todo: in add_position(), using position.locked_amount but the locked_amount can be in different custody, need to manage that
    custody.add_position(position, &collateral_token_price, &mut collateral_custody, curtime)?;
    collateral_custody.update_borrow_rate(curtime)?; 

    Ok(())
}
