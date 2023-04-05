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

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let pda_account = &mut ctx.accounts.pda_account; //todo: check if we have to specify "mut" in #account
        pda_account.is_initialized = true;
        // let pda_account = ctx.accounts.pda_account.as_mut();
        // let bump = pda_account.bump;
        msg!("Account Initialized");
        Ok(())
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        msg!("deposit into limit order pda");

        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.owner_token_vault.to_account_info(),
                    to: ctx.accounts.pda_token_vault.to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
                },
            ),
            amount,
        )?;

        Ok(())
    }

    pub fn process_market_order(
        ctx: Context<ProcessMarketOrder>,
        params: OpenPositionParams,
    ) -> Result<()> {
        msg!("Check CPI 1 ");

        // let pda_account = ctx.accounts.pda_account.as_mut();
        // let bump = pda_account.bump;

        let x = Pubkey::find_program_address(
            &["PdaAccount".as_ref(), ctx.accounts.user.key.as_ref()],
            &limit_order_cpi::id(),
        );
        let bump = x.1;
        msg!("Check CPI 2.1 {:?}", bump);

        let authority_seeds: &[&[&[u8]]] =
            &[&[b"PdaAccount", ctx.accounts.user.key.as_ref(), &[bump]]];

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

#[account]
pub struct UserLimitOrderPdaData {
    pub is_initialized: bool,
    pub owner: Pubkey,
    pub bump: u8,
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        init,
        seeds = [b"PdaAccount".as_ref(), user.key().as_ref()],
        bump,
        payer = user,
        space = 8 + std::mem::size_of::<UserLimitOrderPdaData>()
    )]
    pub pda_account: Box<Account<'info, UserLimitOrderPdaData>>,
    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        mut,
        // has_one = pda_account,
        seeds = [b"PdaAccount".as_ref()],
        bump = pda_account.bump,
    )]
    pub pda_account: Box<Account<'info, UserLimitOrderPdaData>>,
    #[account()]
    pub token_mint: Box<Account<'info, Mint>>,
    #[account(
        init_if_needed,
        payer = user,
        token::mint = token_mint,
        token::authority = user,
        // seeds = [b"limit_order_token_account",
        //          pool.key().as_ref(),
        //          custody_token_mint.key().as_ref()],
        // bump
        // associated_token::mint = token_a_mint,
        // associated_token::authority = swap_account
    )]
    pub owner_token_vault: Box<Account<'info, TokenAccount>>,
    // pub associated_token_program: Program<'info, AssociatedToken>,
    #[account(
        init_if_needed,
        payer = user,
        token::mint = token_mint,
        token::authority = pda_account,
        // seeds = [b"pda_token_account",
        //          pda_account.key().as_ref(),
        //          token_mint.key().as_ref()],
        // bump
    )]
    pub pda_token_vault: Box<Account<'info, TokenAccount>>,
    pub token_program: Program<'info, Token>,
    system_program: Program<'info, System>,
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

    #[account()]
    /// CHECK: non signer user account for Accountinfo in CPI
    pub user: UncheckedAccount<'info>,

    #[account(
        mut,
        // has_one = user,
        // seeds = [b"PdaAccount".as_ref(), user.key().as_ref()],
        // bump = pda_account.bump,
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
