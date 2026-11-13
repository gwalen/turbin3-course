mod common;

use anchor_lang::prelude::Clock;
use anchor_spl::{associated_token::get_associated_token_address, token_interface::TokenAccount};
use common::{
    client::EscrowClient,
    env::INITIAL_TIMESTAMP,
    load, send_tx_expect_error, send_tx_expect_ok,
    token::{TokenFixture, INITIAL_TOKEN_BALANCE},
    Env, DEPOSIT, RECEIVE, SEED,
};
use escrow::state::timed_escrow::TimedEscrow;
use solana_signer::Signer;

#[test]
fn take_expired_swaps_tokens_one_second_before_expiry() {
    let mut env = Env::new();
    let expires_at = INITIAL_TIMESTAMP + 100;
    let token_fixture = TokenFixture::new(&mut env);
    let escrow_client = EscrowClient::new();
    assert!(env.svm.get_account(&token_fixture.taker_ata_a).is_none());
    assert!(env.svm.get_account(&token_fixture.maker_ata_b).is_none());

    let make_timed = escrow_client.make_timed(
        env.maker.pubkey(),
        token_fixture.mint_a,
        token_fixture.mint_b,
        SEED,
        DEPOSIT,
        RECEIVE,
        expires_at,
    );

    send_tx_expect_ok(&mut env.svm, make_timed, &[&env.maker]);
    let (timed_escrow_address, _) = TimedEscrow::find_pda(&env.maker.pubkey(), SEED);
    let vault_address = get_associated_token_address(&timed_escrow_address, &token_fixture.mint_a);
    let timed_escrow_rent = env.svm.get_balance(&timed_escrow_address).unwrap();
    let vault_rent = env.svm.get_balance(&vault_address).unwrap();
    let maker_lamports_before = env.svm.get_balance(&env.maker.pubkey()).unwrap();

    let mut clock: Clock = env.svm.get_sysvar();
    clock.unix_timestamp = expires_at - 1;
    env.svm.set_sysvar(&clock);

    let take_expired = escrow_client.take_expired(
        env.maker.pubkey(),
        env.taker.pubkey(),
        token_fixture.mint_a,
        token_fixture.mint_b,
        SEED,
    );

    send_tx_expect_ok(&mut env.svm, take_expired, &[&env.taker]);
    assert_eq!(
        load::<TokenAccount>(&env, &token_fixture.taker_ata_a).amount,
        DEPOSIT
    );
    assert_eq!(
        load::<TokenAccount>(&env, &token_fixture.maker_ata_b).amount,
        RECEIVE
    );
    assert_eq!(
        load::<TokenAccount>(&env, &token_fixture.taker_ata_b).amount,
        INITIAL_TOKEN_BALANCE - RECEIVE
    );
    assert_eq!(
        load::<TokenAccount>(&env, &token_fixture.maker_ata_a).amount,
        INITIAL_TOKEN_BALANCE - DEPOSIT
    );
    assert!(env.svm.get_account(&timed_escrow_address).is_none());
    assert!(env.svm.get_account(&vault_address).is_none());
    assert_eq!(
        env.svm.get_balance(&env.maker.pubkey()).unwrap(),
        maker_lamports_before + timed_escrow_rent + vault_rent
    );
}

#[test]
fn take_expired_rejects_the_expiry_boundary_without_moving_tokens() {
    let mut env = Env::new();
    let token_fixture = TokenFixture::new(&mut env);
    let escrow_client = EscrowClient::new();
    let expires_at = INITIAL_TIMESTAMP + 100;

    let make_timed = escrow_client.make_timed(
        env.maker.pubkey(),
        token_fixture.mint_a,
        token_fixture.mint_b,
        SEED,
        DEPOSIT,
        RECEIVE,
        expires_at,
    );

    send_tx_expect_ok(&mut env.svm, make_timed, &[&env.maker]);
    let (timed_escrow_address, _) = TimedEscrow::find_pda(&env.maker.pubkey(), SEED);
    let vault_address = get_associated_token_address(&timed_escrow_address, &token_fixture.mint_a);
    let vault_account_before = env.svm.get_account(&vault_address).unwrap();
    let timed_escrow_account_before = env.svm.get_account(&timed_escrow_address).unwrap();
    let mut clock: Clock = env.svm.get_sysvar();
    clock.unix_timestamp = expires_at;
    env.svm.set_sysvar(&clock);

    let take_expired = escrow_client.take_expired(
        env.maker.pubkey(),
        env.taker.pubkey(),
        token_fixture.mint_a,
        token_fixture.mint_b,
        SEED,
    );

    let error = send_tx_expect_error(&mut env.svm, take_expired, &[&env.taker]);
    assert!(error.contains("Error Code: Expired"), "{error}");
    assert_eq!(
        env.svm.get_account(&vault_address).unwrap(),
        vault_account_before
    );
    assert_eq!(
        env.svm.get_account(&timed_escrow_address).unwrap(),
        timed_escrow_account_before
    );
    assert_eq!(
        load::<TokenAccount>(&env, &token_fixture.taker_ata_b).amount,
        INITIAL_TOKEN_BALANCE
    );
    assert!(env.svm.get_account(&token_fixture.taker_ata_a).is_none());
    assert!(env.svm.get_account(&token_fixture.maker_ata_b).is_none());

    let refund_expired =
        escrow_client.refund_expired(env.maker.pubkey(), token_fixture.mint_a, SEED);

    send_tx_expect_ok(&mut env.svm, refund_expired, &[&env.maker]);

    let take_expired = escrow_client.take_expired(
        env.maker.pubkey(),
        env.taker.pubkey(),
        token_fixture.mint_a,
        token_fixture.mint_b,
        SEED,
    );

    let error = send_tx_expect_error(&mut env.svm, take_expired, &[&env.taker]);
    assert!(
        error.contains("Error Code: AccountNotInitialized"),
        "{error}"
    );
}
