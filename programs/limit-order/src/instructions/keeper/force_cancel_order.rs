//! FillOrder instruction handler

use {
    crate::{
        constant::*,
        error::LimitOrderError,
        math,
        state::{limit_order::*, order::*},
    },
    anchor_lang::prelude::*,
    anchor_spl::token::{Mint, Token, TokenAccount},
    perpetuals::{
        instructions::{
            close_position::ClosePositionParams, get_exit_price_and_fee::GetExitPriceAndFeeParams,
        },
        state::perpetuals::*,
    },
};

#[derive(Accounts)]
pub struct ForceCancelOrder<'info> {
    #[account(mut)]
    pub keeper: Signer<'info>,
    /// CHECK: owner address is checked in the perpetuals
    #[account(mut)]
    pub owner: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [LIMIT_ORDER_TAG],
        bump = limit_order.bump,
        has_one = keeper
    )]
    pub limit_order: Box<Account<'info, LimitOrder>>,

    #[account(
        mut,
        seeds = [ORDER_TAG, owner.key().as_ref(), order.mint.as_ref(), &[order.side as u8], order.price.to_be_bytes().as_ref()],
        bump = order.bump,
        has_one = owner,
        has_one = receiving_custody,
    )]
    pub order: Box<Account<'info, Order>>,

    pub mint: Box<Account<'info, Mint>>,
    #[account(mut)]
    pub receiving_account: Box<Account<'info, TokenAccount>>,

    /// CHECK: perpetuals checks this
    pub transfer_authority: AccountInfo<'info>,
    /// CHECK: perpetuals checks this
    #[account(mut)]
    pub pool: AccountInfo<'info>,
    /// CHECK: perpetuals checks this
    #[account(mut)]
    pub position: AccountInfo<'info>,
    /// CHECK: perpetuals checks this
    #[account(mut)]
    pub receiving_custody: AccountInfo<'info>,
    /// CHECK: perpetuals checks this
    pub receiving_custody_oracle_account: AccountInfo<'info>,
    /// CHECK: perpetuals checks this
    #[account(mut)]
    pub receiving_custody_token_account: AccountInfo<'info>,

    /// CHECK: perpetuals checks this
    pub perpetuals_pda: AccountInfo<'info>,
    /// CHECK: perpetuals checks this
    pub perpetuals: Program<'info, Perpetuals>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
    pub clock: Sysvar<'info, Clock>,
}
impl<'info> ForceCancelOrder<'info> {
    pub fn to_close_position_context(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, perpetuals::cpi::accounts::ClosePosition<'info>> {
        CpiContext::new(
            self.perpetuals.to_account_info(),
            perpetuals::cpi::accounts::ClosePosition {
                signer: self.keeper.to_account_info(),
                owner: self.owner.to_account_info(),
                keeper: self.keeper.to_account_info(),
                receiving_account: self.receiving_account.to_account_info(),
                transfer_authority: self.transfer_authority.to_account_info(),
                perpetuals: self.perpetuals_pda.to_account_info(),
                pool: self.pool.to_account_info(),
                position: self.position.to_account_info(),
                custody: self.receiving_custody.to_account_info(),
                custody_oracle_account: self.receiving_custody_oracle_account.to_account_info(),
                custody_token_account: self.receiving_custody_token_account.to_account_info(),
                token_program: self.token_program.to_account_info(),
            },
        )
    }
    pub fn to_get_exit_price_and_fee_context(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, perpetuals::cpi::accounts::GetExitPriceAndFee<'info>> {
        CpiContext::new(
            self.perpetuals.to_account_info(),
            perpetuals::cpi::accounts::GetExitPriceAndFee {
                perpetuals: self.perpetuals_pda.to_account_info(),
                pool: self.pool.to_account_info(),
                position: self.position.to_account_info(),
                custody: self.receiving_custody.to_account_info(),
                custody_oracle_account: self.receiving_custody_oracle_account.to_account_info(),
            },
        )
    }
}
#[derive(AnchorSerialize, AnchorDeserialize, Copy, Clone)]
pub struct ForceCancelOrderParams {}

pub fn force_cancel_order(
    ctx: Context<ForceCancelOrder>,
    _params: &ForceCancelOrderParams,
) -> Result<()> {
    let order = ctx.accounts.order.as_mut();
    ctx.accounts.limit_order.num_active_order =
        math::checked_sub(ctx.accounts.limit_order.num_active_order, 1u64)?;

    let curtime = ctx.accounts.clock.unix_timestamp;
    // let order_type = order.order_type;
    // let order_side = order.side;
    // let order_price = order.price;
    let order_expiry_time = order.expiry_time;
    let exit_price_and_fee = perpetuals::cpi::get_exit_price_and_fee(
        ctx.accounts.to_get_exit_price_and_fee_context(),
        GetExitPriceAndFeeParams {},
    )?
    .get();
    let exit_price = exit_price_and_fee.price;

    require_gte!(curtime, order_expiry_time, LimitOrderError::OrderNotExpired);

    //update limit_order
    ctx.accounts.limit_order.num_active_order =
        math::checked_sub(ctx.accounts.limit_order.num_active_order, 1u64)?;

    // update total amount
    // todo: write code to remove collateral from total_amounts_data of limi_order
    
    // we can add `remove collateral` before close_position
    
    perpetuals::cpi::close_position(
        ctx.accounts.to_close_position_context(),
        ClosePositionParams { price: exit_price },
    )?;

    Ok(())
}
