use {
    crate::utils::{self, pda},
    anchor_lang::{prelude::Pubkey, ToAccountMetas},
    bonfida_test_utils::ProgramTestContextExt,
    limit_order::{
        instructions::OpenOrderParams,
        state::{limit_order::LimitOrder, order::Order},
    },
    perpetuals::{
        instructions::OpenPositionParams,
        state::{
            custody::Custody,
            perpetuals::Perpetuals,
            position::{Position, Side},
        },
    },
    solana_program_test::{BanksClientError, ProgramTestContext},
    solana_sdk::signer::{keypair::Keypair, Signer},
};

pub async fn test_open_order(
    program_test_ctx: &mut ProgramTestContext,
    owner: &Keypair,
    keeper: &Keypair,
    payer: &Keypair,
    pool_pda: &Pubkey,
    custody_token_mint: &Pubkey,
    dispensing_custody_token_mint: &Pubkey,
    params: OpenOrderParams,
) -> std::result::Result<(), BanksClientError> {
    // ==== WHEN ==============================================================

    // Prepare PDA and addresses
    let transfer_authority_pda = pda::get_transfer_authority_pda().0;
    let perpetuals_pda = pda::get_perpetuals_pda().0;
    let custody_pda = pda::get_custody_pda(pool_pda, custody_token_mint).0;
    let dispensing_custody_pda = pda::get_custody_pda(pool_pda, dispensing_custody_token_mint).0;
    let custody_token_account_pda =
        pda::get_custody_token_account_pda(pool_pda, custody_token_mint).0;

    let (position_pda, position_bump) =
        pda::get_position_pda(&owner.pubkey(), pool_pda, &custody_pda, params.side);

    let funding_account_address =
        utils::find_associated_token_account(&owner.pubkey(), custody_token_mint).0;

    let custody_account = utils::get_account::<Custody>(program_test_ctx, custody_pda).await;
    let custody_oracle_account_address = custody_account.oracle.oracle_account;

    // Save account state before tx execution
    let owner_funding_account_before = program_test_ctx
        .get_token_account(funding_account_address)
        .await
        .unwrap();
    let custody_token_account_before = program_test_ctx
        .get_token_account(custody_token_account_pda)
        .await
        .unwrap();

    let limit_order_pda = pda::get_limit_order_pda().0;
    let (order_pda, order_bump) = pda::get_order_pda(
        &owner.pubkey(),
        custody_token_mint,
        params.side as u8,
        params.price,
    );

    utils::create_and_execute_limit_order_ix(
        program_test_ctx,
        limit_order::accounts::OpenOrder {
            owner: owner.pubkey(),
            keeper: keeper.pubkey(),
            payer: payer.pubkey(),
            limit_order: limit_order_pda,
            order: order_pda,
            receiving_custody: custody_pda,
            dispensing_custody: dispensing_custody_pda,
            mint: *custody_token_mint,
            funding_account: funding_account_address,

            transfer_authority: transfer_authority_pda,
            pool: *pool_pda,
            position: position_pda,
            receiving_custody_oracle_account: custody_oracle_account_address,
            receiving_custody_token_account: custody_token_account_pda,

            perpetuals_pda,
            perpetuals: perpetuals::ID,
            system_program: anchor_lang::system_program::ID,
            token_program: anchor_spl::token::ID,
            rent: solana_program::sysvar::rent::ID,
            clock: solana_program::sysvar::clock::ID,
        }
        .to_account_metas(None),
        limit_order::instruction::OpenOrder { params },
        Some(&payer.pubkey()),
        &[owner, payer],
    )
    .await?;

    // ==== THEN ==============================================================
    // Check the balance change
    {
        let owner_funding_account_after = program_test_ctx
            .get_token_account(funding_account_address)
            .await
            .unwrap();
        let custody_token_account_after = program_test_ctx
            .get_token_account(custody_token_account_pda)
            .await
            .unwrap();

        assert!(owner_funding_account_after.amount < owner_funding_account_before.amount);
        assert!(custody_token_account_after.amount > custody_token_account_before.amount);
    }

    // Check the position
    {
        let position_account = utils::get_account::<Position>(program_test_ctx, position_pda).await;
        let perpetuals_account =
            utils::get_account::<Perpetuals>(program_test_ctx, perpetuals_pda).await;

        assert_eq!(position_account.owner, owner.pubkey());
        assert_eq!(position_account.pool, *pool_pda);
        assert_eq!(position_account.custody, custody_pda);
        assert_eq!(
            position_account.open_time,
            perpetuals_account.inception_time
        );
        assert_eq!(position_account.update_time, 0);
        assert_eq!(position_account.side, params.side);
        assert_eq!(position_account.unrealized_profit_usd, 0);
        assert_eq!(position_account.unrealized_loss_usd, 0);
        assert_eq!(position_account.collateral_amount, params.collateral);
        assert_eq!(position_account.bump, position_bump);
    }

    // Check the order
    {
        let order_account = utils::get_account::<Order>(program_test_ctx, order_pda).await;
        let limit_order_account =
            utils::get_account::<LimitOrder>(program_test_ctx, limit_order_pda).await;

        assert_eq!(order_account.owner, owner.pubkey());
        assert_eq!(order_account.receiving_custody, custody_pda);
        assert_eq!(
            order_account.expiry_time,
            params.expiry_period + order_account.order_time
        );
        assert_eq!(order_account.side, params.side);
        assert_eq!(order_account.order_type, params.order_type);
        assert_eq!(order_account.size, params.size);
        assert_eq!(order_account.bump, order_bump);
    }

    Ok(())
}
