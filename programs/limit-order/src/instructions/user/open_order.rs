//! OpenOrder instruction handler

use {
    crate::{
        constant::*,
        math,
        state::{limit_order::*, order::*},
    },
    anchor_lang::prelude::*,
    anchor_spl::token::{Mint, Token, TokenAccount},
    perpetuals::{
        instructions::open_position::OpenPositionParams,
        state::{perpetuals::*, position::Side},
    },
};

#[derive(Accounts)]
#[instruction(params: OpenOrderParams)]
pub struct OpenOrder<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(mut)]
    pub payer: Signer<'info>,
    /// CHECK: keeper address
    #[account(mut)]
    pub keeper: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [LIMIT_ORDER_TAG],
        bump = limit_order.bump,
        has_one = keeper
    )]
    pub limit_order: Box<Account<'info, LimitOrder>>,

    #[account(
        init,
        seeds = [ORDER_TAG, owner.key().as_ref(), mint.key().as_ref(), &[params.side as u8], params.price.to_be_bytes().as_ref()],
        bump,
        payer = payer,
        space = Order::LEN
    )]
    pub order: Box<Account<'info, Order>>,

    pub mint: Box<Account<'info, Mint>>,
    #[account(mut)]
    pub funding_account: Box<Account<'info, TokenAccount>>,

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
    #[account(mut)]
    pub dispensing_custody: AccountInfo<'info>,
    /// CHECK: perpetuals checks this
    pub perpetuals_pda: AccountInfo<'info>,
    /// CHECK: perpetuals checks this
    pub perpetuals: Program<'info, Perpetuals>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
    pub clock: Sysvar<'info, Clock>,
}
impl<'info> OpenOrder<'info> {
    pub fn to_open_position_context(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, perpetuals::cpi::accounts::OpenPosition<'info>> {
        CpiContext::new(
            self.perpetuals.to_account_info(),
            perpetuals::cpi::accounts::OpenPosition {
                owner: self.owner.to_account_info(),
                payer: self.payer.to_account_info(),
                funding_account: self.funding_account.to_account_info(),
                transfer_authority: self.transfer_authority.to_account_info(),
                perpetuals: self.perpetuals_pda.to_account_info(),
                pool: self.pool.to_account_info(),
                position: self.position.to_account_info(),
                custody: self.receiving_custody.to_account_info(),
                custody_oracle_account: self.receiving_custody_oracle_account.to_account_info(),
                custody_token_account: self.receiving_custody_token_account.to_account_info(),
                system_program: self.system_program.to_account_info(),
                token_program: self.token_program.to_account_info(),
            },
        )
    }
}
#[derive(AnchorSerialize, AnchorDeserialize, Copy, Clone)]
pub struct OpenOrderParams {
    pub side: Side,
    pub price: u64,
    pub trigger_price: u64,
    pub limit_price: u64,
    pub collateral: u64,
    pub size: u64,
    pub expiry_period: i64,
    pub order_type: OrderType,
}

pub fn open_order(ctx: Context<OpenOrder>, params: &OpenOrderParams) -> Result<()> {
    ctx.accounts.order.bump = *ctx.bumps.get("order").unwrap();
    ctx.accounts.order.owner = ctx.accounts.owner.key();
    ctx.accounts.order.mint = ctx.accounts.mint.key();
    ctx.accounts.order.side = params.side;
    ctx.accounts.order.price = params.price;
    ctx.accounts.order.trigger_price = params.trigger_price;
    ctx.accounts.order.limit_price = params.limit_price;
    ctx.accounts.order.funding_account = ctx.accounts.funding_account.key();
    ctx.accounts.order.receiving_custody = ctx.accounts.receiving_custody.key();
    ctx.accounts.order.dispensing_custody = ctx.accounts.dispensing_custody.key();
    ctx.accounts.order.collateral = params.collateral;
    ctx.accounts.order.size = params.size;
    ctx.accounts.order.order_time = ctx.accounts.clock.unix_timestamp;
    ctx.accounts.order.expiry_time = ctx.accounts.clock.unix_timestamp + params.expiry_period;
    ctx.accounts.order.order_type = params.order_type;

    //update num_active_order of limit_order
    ctx.accounts.limit_order.num_active_order =
        math::checked_add(ctx.accounts.limit_order.num_active_order, 1u64)?;

    // update total amount
    // todo: write code to add collateral to total_amounts_data of limi_order

    //pay keeper fee (execution fee)
    LimitOrder::transfer_sol(
        ctx.accounts.payer.to_account_info(),
        ctx.accounts.keeper.to_account_info(),
        ctx.accounts.system_program.to_account_info(),
        ctx.accounts.limit_order.execution_fee,
    )?;

    perpetuals::cpi::open_position(
        ctx.accounts.to_open_position_context(),
        OpenPositionParams {
            price: params.price,
            collateral: params.collateral,
            size: params.size,
            side: params.side,
        },
    )?;

    Ok(())
}
