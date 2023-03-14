//! GetLiquidityFee instruction handler

use {
    crate::state::{
        custody::Custody, oracle::OraclePrice, perpetuals::LiquidityFee, perpetuals::Perpetuals,
        pool::Pool,
    },
    anchor_lang::prelude::*,
    anchor_spl::token::TokenAccount,
};

#[derive(Accounts)]
pub struct GetLiquidityFee<'info> {
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

    #[account(
        mut,
        seeds = [b"custody_token_account",
                 pool.key().as_ref(),
                 custody.mint.as_ref()],
        bump = custody.token_account_bump
    )]
    pub custody_token_account: Box<Account<'info, TokenAccount>>,

    /// CHECK: oracle account for the collateral token
    #[account(
        constraint = custody_oracle_account.key() == custody.oracle.oracle_account
    )]
    pub custody_oracle_account: AccountInfo<'info>,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct GetLiquidityFeeParams {
    amount: u64,
}

pub fn get_liquidity_fee(
    ctx: Context<GetLiquidityFee>,
    params: &GetLiquidityFeeParams,
) -> Result<LiquidityFee> {
    let perpetuals = ctx.accounts.perpetuals.as_mut();
    let pool = ctx.accounts.pool.as_mut();
    let custody = ctx.accounts.custody.as_mut();
    let token_id = pool.get_token_id(&custody.key())?;
    let curtime = perpetuals.get_time()?;

    let token_price = OraclePrice::new_from_oracle(
        custody.oracle.oracle_type,
        &ctx.accounts.custody_oracle_account.to_account_info(),
        custody.oracle.max_price_error,
        custody.oracle.max_price_age_sec,
        curtime,
        false,
    )?;

    let add_liquidity_fee =
        pool.get_add_liquidity_fee(token_id, params.amount, custody, &token_price)?;

    let remove_liquidity_fee =
        pool.get_remove_liquidity_fee(token_id, params.amount, custody, &token_price)?;

    Ok(LiquidityFee {
        add_liquidity_fee,
        remove_liquidity_fee,
    })
}
