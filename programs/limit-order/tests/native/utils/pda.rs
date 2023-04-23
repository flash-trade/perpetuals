use {perpetuals::state::position::Side, solana_sdk::pubkey::Pubkey};

pub fn get_multisig_pda() -> (Pubkey, u8) {
    Pubkey::find_program_address(&["multisig".as_ref()], &perpetuals::id())
}

pub fn get_transfer_authority_pda() -> (Pubkey, u8) {
    Pubkey::find_program_address(&["transfer_authority".as_ref()], &perpetuals::id())
}

pub fn get_perpetuals_pda() -> (Pubkey, u8) {
    Pubkey::find_program_address(&["perpetuals".as_ref()], &perpetuals::id())
}

pub fn get_program_data_pda() -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[perpetuals::id().as_ref()],
        &solana_program::bpf_loader_upgradeable::id(),
    )
}

pub fn get_pool_pda(name: String) -> (Pubkey, u8) {
    Pubkey::find_program_address(&["pool".as_ref(), name.as_bytes()], &perpetuals::id())
}

pub fn get_lp_token_mint_pda(pool_pda: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &["lp_token_mint".as_ref(), pool_pda.as_ref()],
        &perpetuals::id(),
    )
}

pub fn get_custody_pda(pool_pda: &Pubkey, custody_token_mint: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            "custody".as_ref(),
            pool_pda.as_ref(),
            custody_token_mint.as_ref(),
        ],
        &perpetuals::id(),
    )
}

pub fn get_position_pda(
    owner: &Pubkey,
    pool_pda: &Pubkey,
    custody_pda: &Pubkey,
    side: Side,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            "position".as_ref(),
            owner.as_ref(),
            pool_pda.as_ref(),
            custody_pda.as_ref(),
            &[side as u8],
        ],
        &perpetuals::id(),
    )
}

pub fn get_custody_token_account_pda(
    pool_pda: &Pubkey,
    custody_token_mint: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            "custody_token_account".as_ref(),
            pool_pda.as_ref(),
            custody_token_mint.as_ref(),
        ],
        &perpetuals::id(),
    )
}

pub fn get_test_oracle_account(pool_pda: &Pubkey, custody_mint: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            "oracle_account".as_ref(),
            pool_pda.as_ref(),
            custody_mint.as_ref(),
        ],
        &perpetuals::id(),
    )
}

pub fn get_limit_order_pda() -> (Pubkey, u8) {
    Pubkey::find_program_address(&["LIMIT_ORDER_TAG".as_ref()], &limit_order::id())
}

pub fn get_order_pda(owner: &Pubkey, mint: &Pubkey, side: u8, price: u64) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            "ORDER_TAG".as_ref(),
            owner.as_ref(),
            mint.as_ref(),
            &[side as u8],
            price.to_be_bytes().as_ref(),
        ],
        &limit_order::id(),
    )
}

pub fn get_limit_order_program_data_pda() -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[limit_order::id().as_ref()],
        &solana_program::bpf_loader_upgradeable::id(),
    )
}
