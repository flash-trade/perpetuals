//! ReallocLimitOrder instruction handler

use {
    crate::{constant::*, state::limit_order::*},
    anchor_lang::prelude::*,
};

#[derive(Accounts)]
pub struct ReallocLimitOrder<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        mut,
        seeds = [
            LIMIT_ORDER_TAG
        ],
        bump = limit_order.bump,
        realloc = LimitOrder::LEN + limit_order.total_amounts_data.cur_max_total_amounts_possable as usize * TotalAmount::LEN,
        realloc::payer = payer,
        realloc::zero = false,
        has_one = admin,
        constraint = limit_order.total_amounts_data.cur_max_total_amounts_possable < TotalAmountsData::MAX_ACTIVE_TOTAL_AMOUNTS
    )]
    pub limit_order: Box<Account<'info, LimitOrder>>,

    pub system_program: Program<'info, System>,
}

impl<'info> ReallocLimitOrder<'info> {
    pub fn assert_need_resizing(ctx: &Context<ReallocLimitOrder>) -> Result<()> {
        let limit_order = &ctx.accounts.limit_order;
        if !limit_order.total_amounts_data.need_resizing() {
            msg!("ErrorCode::Doesn't need resizing");
            panic!("error");
        }
        Ok(())
    }
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct ReallocLimitOrderParams {}
#[access_control(ReallocLimitOrder::assert_need_resizing(&ctx))]
pub fn reallocate(
    ctx: Context<ReallocLimitOrder>,
    _params: &ReallocLimitOrderParams,
) -> Result<()> {
    let limit_order = &mut ctx.accounts.limit_order;
    limit_order
        .total_amounts_data
        .cur_max_total_amounts_possable += TotalAmountsData::INITIAL_MAX_TOTAL_AMOUNTS;
    Ok(())
}
