use {
    anchor_lang::prelude::*,
    anchor_spl::token::{Burn, MintTo, Transfer},
};

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, Default, Debug)]
pub struct TotalAmount {
    pub mint: Pubkey,
    pub amount: u64,
}

impl TotalAmount {
    pub const LEN: usize = std::mem::size_of::<TotalAmount>();
}
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, Default, Debug)]
pub struct TotalAmountsData {
    pub num_active_total_amounts: u32,
    pub cur_max_total_amounts_possable: u32,
    pub total_amounts: Vec<TotalAmount>,
}
impl TotalAmountsData {
    pub const MAX_ACTIVE_TOTAL_AMOUNTS: u32 = 10000;
    pub const INITIAL_MAX_TOTAL_AMOUNTS: u32 = 20;
    pub const RESIZING_SPACE: usize = Self::INITIAL_MAX_TOTAL_AMOUNTS as usize * TotalAmount::LEN;
    pub const INITIAL_SPACE_FOR_TOTAL_AMOUNTS_DATA: usize = 4 + 4 + 4 + Self::RESIZING_SPACE;
    pub fn new() -> Self {
        Self {
            num_active_total_amounts: 0u32,
            cur_max_total_amounts_possable: Self::INITIAL_MAX_TOTAL_AMOUNTS,
            total_amounts: Vec::<TotalAmount>::new(),
        }
    }
    pub fn get_total_amount(&self, index: usize) -> &TotalAmount {
        &self.total_amounts[index]
    }
    pub fn get_total_amount_mut(&mut self, index: usize) -> &mut TotalAmount {
        &mut self.total_amounts[index]
    }
    fn new_recording_possible(&self) -> bool {
        self.num_active_total_amounts < self.cur_max_total_amounts_possable
    }
    fn resizing_possible(&self) -> bool {
        self.cur_max_total_amounts_possable < Self::MAX_ACTIVE_TOTAL_AMOUNTS
    }
    pub fn need_resizing(&self) -> bool {
        self.resizing_possible() && !self.new_recording_possible()
    }
}
#[account]
#[derive(Default, Debug)]
pub struct LimitOrder {
    pub admin: Pubkey,
    pub keeper: Pubkey,
    pub fee_owner: Pubkey,
    pub num_active_order: u64,
    pub execution_fee: u64,

    pub total_amounts_data: TotalAmountsData,

    pub bump: u8,
}

impl anchor_lang::Id for LimitOrder {
    fn id() -> Pubkey {
        crate::ID
    }
}

impl LimitOrder {
    pub const LEN: usize =
        8 + TotalAmountsData::INITIAL_SPACE_FOR_TOTAL_AMOUNTS_DATA + 32 + 32 + 8 + 8 + 8 + 8 + 1;

    pub fn is_empty_account(account_info: &AccountInfo) -> Result<bool> {
        Ok(account_info.try_data_is_empty()? || account_info.try_lamports()? == 0)
    }

    pub fn close_token_account<'info>(
        receiver: AccountInfo<'info>,
        token_account: AccountInfo<'info>,
        token_program: AccountInfo<'info>,
        authority: AccountInfo<'info>,
        seeds: &[&[&[u8]]],
    ) -> Result<()> {
        let cpi_accounts = anchor_spl::token::CloseAccount {
            account: token_account,
            destination: receiver,
            authority,
        };
        let cpi_context = anchor_lang::context::CpiContext::new(token_program, cpi_accounts);

        anchor_spl::token::close_account(cpi_context.with_signer(seeds))
    }

    pub fn transfer_sol_from_owned<'a>(
        program_owned_source_account: AccountInfo<'a>,
        destination_account: AccountInfo<'a>,
        amount: u64,
    ) -> Result<()> {
        **destination_account.try_borrow_mut_lamports()? = destination_account
            .try_lamports()?
            .checked_add(amount)
            .ok_or(ProgramError::InsufficientFunds)?;

        let source_balance = program_owned_source_account.try_lamports()?;
        **program_owned_source_account.try_borrow_mut_lamports()? = source_balance
            .checked_sub(amount)
            .ok_or(ProgramError::InsufficientFunds)?;

        Ok(())
    }

    pub fn transfer_sol<'a>(
        source_account: AccountInfo<'a>,
        destination_account: AccountInfo<'a>,
        system_program: AccountInfo<'a>,
        amount: u64,
    ) -> Result<()> {
        let cpi_accounts = anchor_lang::system_program::Transfer {
            from: source_account,
            to: destination_account,
        };
        let cpi_context = anchor_lang::context::CpiContext::new(system_program, cpi_accounts);

        anchor_lang::system_program::transfer(cpi_context, amount)
    }

    pub fn realloc<'a>(
        funding_account: AccountInfo<'a>,
        target_account: AccountInfo<'a>,
        system_program: AccountInfo<'a>,
        new_len: usize,
        zero_init: bool,
    ) -> Result<()> {
        let new_minimum_balance = Rent::get()?.minimum_balance(new_len);
        let lamports_diff = new_minimum_balance.saturating_sub(target_account.try_lamports()?);

        LimitOrder::transfer_sol(
            funding_account,
            target_account.clone(),
            system_program,
            lamports_diff,
        )?;

        target_account
            .realloc(new_len, zero_init)
            .map_err(|_| ProgramError::InvalidRealloc.into())
    }
}
