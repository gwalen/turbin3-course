mod common;

use anchor_spl::{associated_token::get_associated_token_address, token_interface::TokenAccount};
use common::{
    client::EscrowClient,
    load, send_tx_expect_error, send_tx_expect_ok,
    token::{TokenFixture, INITIAL_TOKEN_BALANCE},
    Env, DEPOSIT, RECEIVE, SEED,
};
use escrow::state::escrow::Escrow;
use solana_signer::Signer;

#[test]
fn take_swaps_tokens_creates_missing_atas_and_returns_both_rents_to_maker() {
    let mut env = Env::new();
    let tokens = TokenFixture::new(&mut env);
    let client = EscrowClient::new();
    assert!(env.svm.get_account(&tokens.taker_ata_a).is_none());
    assert!(env.svm.get_account(&tokens.maker_ata_b).is_none());

    let make = client.make(
        env.maker.pubkey(),
        tokens.mint_a,
        tokens.mint_b,
        SEED,
        DEPOSIT,
        RECEIVE,
    );

    send_tx_expect_ok(&mut env.svm, make, &[&env.maker]);
    let (escrow_address, _) = Escrow::find_pda(&env.maker.pubkey(), SEED);
    let vault_address = get_associated_token_address(&escrow_address, &tokens.mint_a);
    let escrow_rent = env.svm.get_balance(&escrow_address).unwrap();
    let vault_rent = env.svm.get_balance(&vault_address).unwrap();
    let maker_lamports_before = env.svm.get_balance(&env.maker.pubkey()).unwrap();

    let take = client.take(
        env.maker.pubkey(),
        env.taker.pubkey(),
        tokens.mint_a,
        tokens.mint_b,
        SEED,
    );

    send_tx_expect_ok(&mut env.svm, take, &[&env.taker]);
    assert_eq!(
        load::<TokenAccount>(&env, &tokens.taker_ata_a).amount,
        DEPOSIT
    );
    assert_eq!(
        load::<TokenAccount>(&env, &tokens.maker_ata_b).amount,
        RECEIVE
    );
    assert_eq!(
        load::<TokenAccount>(&env, &tokens.taker_ata_b).amount,
        INITIAL_TOKEN_BALANCE - RECEIVE
    );
    assert_eq!(
        load::<TokenAccount>(&env, &tokens.maker_ata_a).amount,
        INITIAL_TOKEN_BALANCE - DEPOSIT
    );
    assert!(env.svm.get_account(&escrow_address).is_none());
    assert!(env.svm.get_account(&vault_address).is_none());
    assert_eq!(
        env.svm.get_balance(&env.maker.pubkey()).unwrap(),
        maker_lamports_before + escrow_rent + vault_rent
    );
}

#[test]
fn taking_one_escrow_leaves_the_other_offer_unchanged() {
    let mut env = Env::new();
    let tokens = TokenFixture::new(&mut env);
    let client = EscrowClient::new();
    for seed in [SEED, SEED + 1] {
        let make = client.make(
            env.maker.pubkey(),
            tokens.mint_a,
            tokens.mint_b,
            seed,
            DEPOSIT,
            RECEIVE,
        );

        send_tx_expect_ok(&mut env.svm, make, &[&env.maker]);
    }
    let (remaining_escrow_address, _) = Escrow::find_pda(&env.maker.pubkey(), SEED + 1);
    let remaining_escrow_before = env.svm.get_account(&remaining_escrow_address).unwrap();
    let remaining_vault_address =
        get_associated_token_address(&remaining_escrow_address, &tokens.mint_a);
    let remaining_vault_before = env.svm.get_account(&remaining_vault_address).unwrap();

    let take = client.take(
        env.maker.pubkey(),
        env.taker.pubkey(),
        tokens.mint_a,
        tokens.mint_b,
        SEED,
    );

    send_tx_expect_ok(&mut env.svm, take, &[&env.taker]);
    assert_eq!(
        env.svm.get_account(&remaining_escrow_address).unwrap(),
        remaining_escrow_before
    );
    assert_eq!(
        env.svm.get_account(&remaining_vault_address).unwrap(),
        remaining_vault_before
    );
}

#[test]
fn insufficient_payment_rolls_back_tokens_and_new_accounts() {
    use anchor_spl::token::{self, spl_token::instruction::burn};
    let mut env = Env::new();
    let tokens = TokenFixture::new(&mut env);
    let client = EscrowClient::new();

    let burn_payment = burn(
        &token::ID,
        &tokens.taker_ata_b,
        &tokens.mint_b,
        &env.taker.pubkey(),
        &[],
        INITIAL_TOKEN_BALANCE - RECEIVE + 1,
    )
    .unwrap();

    send_tx_expect_ok(&mut env.svm, burn_payment, &[&env.taker]);

    let make = client.make(
        env.maker.pubkey(),
        tokens.mint_a,
        tokens.mint_b,
        SEED,
        DEPOSIT,
        RECEIVE,
    );

    send_tx_expect_ok(&mut env.svm, make, &[&env.maker]);
    let (escrow_address, _) = Escrow::find_pda(&env.maker.pubkey(), SEED);
    let vault_address = get_associated_token_address(&escrow_address, &tokens.mint_a);
    let vault_before = env.svm.get_account(&vault_address).unwrap();
    let escrow_before = env.svm.get_account(&escrow_address).unwrap();
    let maker_lamports_before = env.svm.get_balance(&env.maker.pubkey()).unwrap();

    let take = client.take(
        env.maker.pubkey(),
        env.taker.pubkey(),
        tokens.mint_a,
        tokens.mint_b,
        SEED,
    );

    let error = send_tx_expect_error(&mut env.svm, take, &[&env.taker]);
    assert!(error.contains("insufficient funds"), "{error}");
    assert_eq!(env.svm.get_account(&vault_address).unwrap(), vault_before);
    assert_eq!(env.svm.get_account(&escrow_address).unwrap(), escrow_before);
    assert_eq!(
        load::<TokenAccount>(&env, &tokens.taker_ata_b).amount,
        RECEIVE - 1
    );
    assert_eq!(
        load::<TokenAccount>(&env, &tokens.maker_ata_a).amount,
        INITIAL_TOKEN_BALANCE - DEPOSIT
    );
    assert!(env.svm.get_account(&tokens.maker_ata_b).is_none());
    assert!(env.svm.get_account(&tokens.taker_ata_a).is_none());
    assert_eq!(
        env.svm.get_balance(&env.maker.pubkey()).unwrap(),
        maker_lamports_before
    );
}
