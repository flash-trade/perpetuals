//! Initialize instruction handler

use {
    crate::{constant::*, error::LimitOrderError, state::limit_order::*},
    anchor_lang::prelude::*,
    anchor_spl::token::Token,
    solana_program::program_error::ProgramError,
};

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        init,
        seeds = [LIMIT_ORDER_TAG],
        bump,
        payer = payer,
        space = LimitOrder::LEN
    )]
    pub limit_order: Box<Account<'info, LimitOrder>>,

    /// limit order program.
    /// Provided here to check the upgrade authority.
    #[account(
        constraint = limit_order_program.programdata_address()? == Some(program_data.key()) @ LimitOrderError::InvalidProgramData
    )]
    pub limit_order_program: Program<'info, LimitOrder>,

    /// The program data account for the limit order program.
    /// Provided to check the upgrade authority.
    #[account(
        constraint = program_data.upgrade_authority_address == Some(admin.key()) @ LimitOrderError::InvalidProgramUpgradeAuthority
    )]
    pub program_data: Account<'info, ProgramData>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Copy, Clone)]
pub struct InitParams {
    pub new_admin: Pubkey,
    pub fee_owner: Pubkey,
    pub keeper: Pubkey,
    pub execution_fee: u64,
}

pub fn initialize(ctx: Context<Initialize>, params: &InitParams) -> Result<()> {
    let limit_order = ctx.accounts.limit_order.as_mut();
    limit_order.admin = params.new_admin;
    limit_order.fee_owner = params.fee_owner;
    limit_order.keeper = params.keeper;
    limit_order.execution_fee = params.execution_fee;
    limit_order.num_active_order = 0u64;
    limit_order.bump = *ctx.bumps.get("limit_order").unwrap();
    limit_order.total_amounts_data = TotalAmountsData::new();

    Ok(())
}
