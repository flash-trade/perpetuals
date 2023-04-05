// use anchor_lang::prelude::*;
// use anchor_lang::solana_program::system_program;
// use anchor_spl::token::{self, Token, TokenAccount, Transfer};

use {
    anchor_lang::prelude::*,
    anchor_lang::solana_program::system_program,
    anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer},
    perpetuals::{
        self,
        cpi::accounts::OpenPosition,
        // instructions::OpenPositionParams,
        // perpetuals::{open_position},
        state::{
            custody::Custody,
            perpetuals::Perpetuals,
            pool::Pool,
            position::Position,
            // position::Side,
        },
    },
};
// use flash::

// use flash::accounts::OpenPosition
// use jupiter;

declare_id!("41Af5KuLs3fQobV1Pn4q39LGw3aDwY9SWQ4Sj5rB4ZjE");

#[program]
mod limit_order_cpi {

    use super::*;

    pub fn process_market_order(
        ctx: Context<ProcessMarketOrder>,
        params: OpenPositionParams,
    ) -> Result<()> {
        msg!("Check CPI 1 ");

        let x = Pubkey::find_program_address(&["PdaDirect1".as_ref()], &limit_order_cpi::id());
        let bump = x.1;
        msg!("Check CPI 2.1 {:?}", bump);

        let authority_seeds: &[&[&[u8]]] = &[&[b"PdaDirect1", &[bump]]];

        let cpi_program = ctx.accounts.flash_program.to_account_info();

        // drop(pda_account);
        msg!("Check CPI 2");

        let cpi_accounts = OpenPosition {
            owner: ctx.accounts.pda_account.to_account_info(),
            funding_account: ctx.accounts.pda_token_vault.to_account_info(),
            transfer_authority: ctx.accounts.transfer_authority.to_account_info(),
            perpetuals: ctx.accounts.perpetuals.to_account_info(),
            pool: ctx.accounts.pool.to_account_info(),
            position: ctx.accounts.position.to_account_info(),
            custody: ctx.accounts.custody.to_account_info(),
            custody_oracle_account: ctx.accounts.custody_oracle_account.to_account_info(),
            custody_token_account: ctx.accounts.custody_token_account.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
            token_program: ctx.accounts.token_program.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts).with_signer(authority_seeds);

        perpetuals::cpi::open_position(
            cpi_ctx,
            perpetuals::instructions::OpenPositionParams {
                price: params.price,
                collateral: params.collateral,
                size: params.size,
                side: perpetuals::state::position::Side::Long,
            },
        )?;

        Ok(())
    }
}

#[derive(Copy, Clone, PartialEq, AnchorSerialize, AnchorDeserialize, Debug)]
pub enum Side {
    None,
    Long,
    Short,
}
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy)]
pub struct OpenPositionParams {
    pub price: u64,
    pub collateral: u64,
    pub size: u64,
    pub side: Side,
}

#[derive(Accounts)]
#[instruction(params: OpenPositionParams)]
pub struct ProcessMarketOrder<'info> {
    #[account(mut)]
    pub keeper: Signer<'info>,

    #[account(
        mut,
        // has_one = user,
        seeds = [b"PdaDirect1".as_ref()],
        bump = 255,
    )]
    /// CHECK: sdssds
    pub pda_account: AccountInfo<'info>,

    // from open_position CPI
    // #[account(
    //     mut,
    //     constraint = funding_account.mint == custody.mint,
    //     has_one = owner
    // )]
    // pub funding_account: Box<Account<'info, TokenAccount>>,
    // here it is limit_order_token_Acc
    #[account(
        mut,
        // constraint = pda_token_vault.mint == custody.mint,
        // has_one = pda_account
    )]
    /// CHECK: sds
    pub pda_token_vault: AccountInfo<'info>,

    /// CHECK: empty PDA, authority for token accounts
    #[account()]
    pub transfer_authority: AccountInfo<'info>,

    /// CHECK: oracle
    #[account()]
    pub perpetuals: AccountInfo<'info>,

    /// CHECK: empty PDA, authority for token accounts
    #[account(mut)]
    pub pool: AccountInfo<'info>,

    // #[account(
    //     init,
    //     payer = keeper,
    //     space = Position::LEN,
    //     seeds = [b"position",
    //             pda_account.owner.key().as_ref(),
    //              pool.key().as_ref(),
    //              custody.key().as_ref(),
    //              &[params.side as u8]],
    //     bump
    // )]
    /// CHECK: oracle
    #[account(mut)]
    pub position: AccountInfo<'info>,

    // #[account(
    //     mut,
    //     seeds = [b"custody",
    //              pool.key().as_ref(),
    //              custody.mint.as_ref()],
    //     bump = custody.bump
    // )]
    /// CHECK: oracle
    #[account(mut)]
    pub custody: AccountInfo<'info>,

    /// CHECK: oracle account for the collateral token
    #[account(
        // constraint = custody_oracle_account.key() == custody.oracle.oracle_account
    )]
    pub custody_oracle_account: AccountInfo<'info>,

    // #[account(
    //     mut,
    //     seeds = [b"custody_token_account",
    //              pool.key().as_ref(),
    //              custody.mint.as_ref()],
    //     bump = custody.token_account_bump
    // )]
    /// CHECK: ora
    #[account(mut)]
    pub custody_token_account: AccountInfo<'info>,

    system_program: Program<'info, System>,
    token_program: Program<'info, Token>,
    flash_program: Program<'info, Perpetuals>,
}
