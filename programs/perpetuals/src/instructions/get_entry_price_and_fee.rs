//! GetEntryPriceAndFee instruction handler

use {
    crate::{
        math,
        state::{
            custody::Custody,
            oracle::OraclePrice,
            perpetuals::{NewPositionPricesAndFee, Perpetuals},
            pool::Pool,
            position::{Position, Side},
        }
    },
    anchor_lang::prelude::*,
    solana_program::program_error::ProgramError,
};

#[derive(Accounts)]
pub struct GetEntryPriceAndFee<'info> {
    #[account(
        seeds = [b"perpetuals"],
        bump = perpetuals.perpetuals_bump
    )]
    pub perpetuals: Box<Account<'info, Perpetuals>>,

    #[account(
        seeds = [b"pool",
                 pool.name.as_bytes()],
        bump = pool.bump
    )]
    pub pool: Box<Account<'info, Pool>>,

    #[account(
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
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct GetEntryPriceAndFeeParams {
    collateral: u64,
    size: u64,
    side: Side,
}

pub fn get_entry_price_and_fee(
    ctx: Context<GetEntryPriceAndFee>,
    params: &GetEntryPriceAndFeeParams,
) -> Result<NewPositionPricesAndFee> {
    // validate inputs
    if params.collateral == 0 || params.size == 0 || params.side == Side::None {
        return Err(ProgramError::InvalidArgument.into());
    }
    let pool = &ctx.accounts.pool;
    let custody = ctx.accounts.custody.as_mut();
    let collateral_custody = ctx.accounts.collateral_custody.as_mut();

    // compute position price
    let curtime = ctx.accounts.perpetuals.get_time()?;

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
        &ctx.accounts.custody_oracle_account.to_account_info(),
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

    let locked_amount = if params.side == Side::Long {
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

    let entry_price = pool.get_entry_price(&token_price, &token_ema_price, params.side, custody)?;

    let size_usd = custody_min_price.get_asset_amount_usd(params.size, custody.decimals)?;
    let collateral_usd = collateral_min_price.get_asset_amount_usd(params.collateral, collateral_custody.decimals)?;

    let position = Position {
        side: params.side,
        price: entry_price,
        size_usd,
        collateral_usd,
        cumulative_interest_snapshot: custody.get_cumulative_interest(curtime)?,
        ..Position::default()
    };
    
    let liquidation_price =
        pool.get_liquidation_price(&position, &token_ema_price, custody, collateral_custody, curtime)?;

    let fee = pool.get_entry_fee(
        params.size,
        locked_amount.try_into().unwrap(), 
        &token_price, 
        &collateral_min_price,
        custody,
        collateral_custody
    )?;

    Ok(NewPositionPricesAndFee {
        entry_price,
        liquidation_price,
        fee,
    })
}
