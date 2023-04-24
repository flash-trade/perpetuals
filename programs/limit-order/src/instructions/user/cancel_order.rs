//! CancelOrder instruction handler

use {
    crate::{
        constant::*,
        math,
        state::{limit_order::*, order::*},
    },
    anchor_lang::prelude::*,
    anchor_spl::token::{Mint, Token, TokenAccount},
    perpetuals::{instructions::close_position::ClosePositionParams, state::perpetuals::*},
};

#[derive(Accounts)]
pub struct CancelOrder<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    /// CHECK: owner address is checked in the perpetuals
    #[account(mut)]
    pub owner: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [LIMIT_ORDER_TAG],
        bump = limit_order.bump,
    )]
    pub limit_order: Box<Account<'info, LimitOrder>>,

    #[account(
        mut,
        seeds = [ORDER_TAG, owner.key().as_ref(), order.mint.as_ref(), &[order.side as u8], order.price.to_be_bytes().as_ref()],
        bump = order.bump,
        has_one = owner,
        has_one = receiving_custody,
        close = owner
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
impl<'info> CancelOrder<'info> {
    pub fn to_close_position_context(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, perpetuals::cpi::accounts::ClosePosition<'info>> {
        CpiContext::new(
            self.perpetuals.to_account_info(),
            perpetuals::cpi::accounts::ClosePosition {
                signer: self.owner.to_account_info(),
                owner: self.owner.to_account_info(),
                keeper: self.owner.to_account_info(),
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
}
#[derive(AnchorSerialize, AnchorDeserialize, Copy, Clone)]
pub struct CancelOrderParams {
    pub price: u64,
}

pub fn cancel_order(ctx: Context<CancelOrder>, params: &CancelOrderParams) -> Result<()> {
    //update limit_order
    ctx.accounts.limit_order.num_active_order =
        math::checked_sub(ctx.accounts.limit_order.num_active_order, 1u64)?;
    // update total amount
    // todo: write code to remove collateral from total_amounts_data of limi_order

    perpetuals::cpi::close_position(
        ctx.accounts.to_close_position_context(),
        ClosePositionParams {
            price: params.price,
        },
    )?;
    Ok(())
}
