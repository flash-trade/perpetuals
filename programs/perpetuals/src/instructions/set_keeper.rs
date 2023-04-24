//! SetKeeper instruction handler

use {
    crate::{
        error::PerpetualsError,
        state::{
            multisig::{AdminInstruction, Multisig},
            perpetuals::Perpetuals,
        },
    },
    anchor_lang::prelude::*,
};

#[derive(Accounts)]
pub struct SetKeeper<'info> {
    #[account()]
    pub admin: Signer<'info>,

    #[account(
        mut,
        seeds = [b"multisig"],
        bump = multisig.load()?.bump
    )]
    pub multisig: AccountLoader<'info, Multisig>,

    #[account(
        mut,
        seeds = [b"perpetuals"],
        bump = perpetuals.perpetuals_bump
    )]
    pub perpetuals: Box<Account<'info, Perpetuals>>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Copy, Clone)]
pub struct SetKeeperParams {
    pub keeper: Pubkey,
}

pub fn set_keeper<'info>(
    ctx: Context<'_, '_, '_, 'info, SetKeeper<'info>>,
    params: &SetKeeperParams,
) -> Result<u8> {
    // validate signatures
    let mut multisig = ctx.accounts.multisig.load_mut()?;

    let signatures_left = multisig.sign_multisig(
        &ctx.accounts.admin,
        &Multisig::get_account_infos(&ctx)[1..],
        &Multisig::get_instruction_data(AdminInstruction::SetKeeper, params)?,
    )?;
    if signatures_left > 0 {
        msg!(
            "Instruction has been signed but more signatures are required: {}",
            signatures_left
        );
        return Ok(signatures_left);
    }

    // update permissions
    let perpetuals = ctx.accounts.perpetuals.as_mut();
    perpetuals.keeper = params.keeper;

    if !perpetuals.validate() {
        err!(PerpetualsError::InvalidPerpetualsConfig)
    } else {
        Ok(0)
    }
}
