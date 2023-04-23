use {
    crate::utils::{self, pda},
    anchor_lang::{
        prelude::{AccountMeta, Pubkey},
        ToAccountMetas,
    },
    perpetuals::{
        instructions::SetKeeperParams,
        state::{custody::Custody, multisig::Multisig, perpetuals::Perpetuals},
    },
    solana_program_test::{BanksClientError, ProgramTestContext},
    solana_sdk::signer::{keypair::Keypair, Signer},
};

pub async fn test_set_keeper(
    program_test_ctx: &mut ProgramTestContext,
    admin: &Keypair,
    payer: &Keypair,
    params: SetKeeperParams,
    multisig_signers: &[&Keypair],
) -> std::result::Result<(), BanksClientError> {
    // ==== WHEN ==============================================================
    let transfer_authority_pda = pda::get_transfer_authority_pda().0;
    let perpetuals_pda = pda::get_perpetuals_pda().0;
    let multisig_pda = pda::get_multisig_pda().0;
    let multisig_account = utils::get_account::<Multisig>(program_test_ctx, multisig_pda).await;

    // One Tx per multisig signer
    for i in 0..multisig_account.min_signatures {
        let signer: &Keypair = multisig_signers[i as usize];

        let accounts_meta = {
            let accounts = perpetuals::accounts::SetKeeper {
                admin: admin.pubkey(),
                multisig: multisig_pda,
                perpetuals: perpetuals_pda,
            };

            let mut accounts_meta = accounts.to_account_metas(None);

            accounts_meta.push(AccountMeta {
                pubkey: signer.pubkey(),
                is_signer: true,
                is_writable: false,
            });

            accounts_meta
        };

        utils::create_and_execute_perpetuals_ix(
            program_test_ctx,
            accounts_meta,
            perpetuals::instruction::SetKeeper {
                params: params.clone(),
            },
            Some(&payer.pubkey()),
            &[admin, payer, signer],
        )
        .await?;
    }

    // ==== THEN ==============================================================
    let perpetuals_account =
        utils::get_account::<Perpetuals>(program_test_ctx, perpetuals_pda).await;

    // Check perpetuals account
    {
        assert_eq!(perpetuals_account.keeper, params.keeper);
    }

    Ok(())
}
