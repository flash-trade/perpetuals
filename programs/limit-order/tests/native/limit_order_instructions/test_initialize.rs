use {
    crate::utils::{self, pda},
    anchor_lang::{prelude::AccountMeta, ToAccountMetas},
    limit_order::{instructions::InitParams, state::limit_order::LimitOrder},
    solana_program_test::{BanksClientError, ProgramTestContext},
    solana_sdk::signer::{keypair::Keypair, Signer},
};

pub async fn test_initialize(
    program_test_ctx: &mut ProgramTestContext,
    params: InitParams,
    admin: &Keypair, //upgrade authority
) -> std::result::Result<(), BanksClientError> {
    // ==== WHEN ==============================================================
    let limit_order_program_data_pda = pda::get_limit_order_program_data_pda().0;
    let (limit_order_pda, limit_order_bump) = pda::get_limit_order_pda();

    let accounts_meta = {
        let accounts = limit_order::accounts::Initialize {
            admin: admin.pubkey(),
            payer: admin.pubkey(),
            limit_order: limit_order_pda,
            limit_order_program: limit_order::ID,
            program_data: limit_order_program_data_pda,
            system_program: anchor_lang::system_program::ID,
            token_program: anchor_spl::token::ID,
            rent: solana_program::sysvar::rent::ID,
        };

        let mut accounts_meta = accounts.to_account_metas(None);

        accounts_meta.push(AccountMeta {
            pubkey: admin.pubkey(),
            is_signer: true,
            is_writable: true,
        });

        accounts_meta
    };

    utils::create_and_execute_limit_order_ix(
        program_test_ctx,
        accounts_meta,
        limit_order::instruction::Initialize { params },
        Some(&admin.pubkey()),
        &[admin],
    )
    .await?;

    // ==== THEN ==============================================================
    let limit_order_account =
        utils::get_account::<LimitOrder>(program_test_ctx, limit_order_pda).await;

    assert_eq!(limit_order_account.admin, params.new_admin);
    assert_eq!(limit_order_account.fee_owner, params.fee_owner);
    assert_eq!(limit_order_account.num_active_order, 0u64);
    assert_eq!(limit_order_account.execution_fee, 1_000_000u64);
    assert_eq!(limit_order_account.bump, limit_order_bump);

    Ok(())
}
