//! Limit order program entrypoint

#![allow(clippy::result_large_err)]

pub mod constant;
pub mod error;
pub mod instructions;
pub mod math;
pub mod state;

use {anchor_lang::prelude::*, instructions::*};

declare_id!("LimtKAwfXxMMD2HgYiNwAXBRTauJZUJG1utKMazsT1m");

#[program]
pub mod limit_order {
    use super::*;
    // admin instructions
    pub fn initialize(ctx: Context<Initialize>, params: InitParams) -> Result<()> {
        instructions::initialize(ctx, &params)
    }
    pub fn reallocate(
        ctx: Context<ReallocLimitOrder>,
        params: ReallocLimitOrderParams,
    ) -> Result<()> {
        instructions::reallocate(ctx, &params)
    }
    pub fn update_config(ctx: Context<UpdateConfig>, params: UpdateConfigParams) -> Result<()> {
        instructions::update_config(ctx, &params)
    }
    // user instructions
    pub fn open_order(ctx: Context<OpenOrder>, params: OpenOrderParams) -> Result<()> {
        instructions::open_order(ctx, &params)
    }
    pub fn cancel_order(ctx: Context<CancelOrder>, params: CancelOrderParams) -> Result<()> {
        instructions::cancel_order(ctx, &params)
    }
    // keeper instructions
    pub fn fill_order(ctx: Context<FillOrder>, params: FillOrderParams) -> Result<()> {
        instructions::fill_order(ctx, &params)
    }
    pub fn force_cancel_order(
        ctx: Context<ForceCancelOrder>,
        params: ForceCancelOrderParams,
    ) -> Result<()> {
        instructions::force_cancel_order(ctx, &params)
    }
}
