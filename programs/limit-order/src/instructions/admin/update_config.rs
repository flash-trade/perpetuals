//! UpdateConfig instruction handler

use {
    crate::{constant::*, state::limit_order::*},
    anchor_lang::prelude::*,
};

#[derive(Accounts)]
pub struct UpdateConfig<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        mut,
        seeds = [LIMIT_ORDER_TAG],
        bump = limit_order.bump,
        has_one = admin
    )]
    pub limit_order: Box<Account<'info, LimitOrder>>,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct UpdateConfigParams {
    pub new_admin: Pubkey,
    pub fee_owner: Pubkey,
    pub keeper: Pubkey,
    pub execution_fee: u64,
}

pub fn update_config(ctx: Context<UpdateConfig>, params: &UpdateConfigParams) -> Result<()> {
    let limit_order = ctx.accounts.limit_order.as_mut();
    limit_order.admin = params.new_admin;
    limit_order.fee_owner = params.fee_owner;
    limit_order.keeper = params.keeper;
    limit_order.execution_fee = params.execution_fee;
    Ok(())
}
