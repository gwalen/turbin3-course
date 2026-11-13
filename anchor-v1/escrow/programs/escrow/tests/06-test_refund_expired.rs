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
fn refund_expired_locks_tokens_until_expiry_then_returns_tokens_and_rents() {
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
    let timed_escrow_account_before = env.svm.get_account(&timed_escrow_address).unwrap();
    let vault_account_before = env.svm.get_account(&vault_address).unwrap();
    for unix_timestamp in [INITIAL_TIMESTAMP, expires_at - 1] {
        let mut clock: Clock = env.svm.get_sysvar();
        clock.unix_timestamp = unix_timestamp;
        env.svm.set_sysvar(&clock);

        let refund_expired =
            escrow_client.refund_expired(env.maker.pubkey(), token_fixture.mint_a, SEED);

        let error = send_tx_expect_error(&mut env.svm, refund_expired, &[&env.maker]);
        assert!(error.contains("Error Code: NotExpired"), "{error}");
        assert_eq!(
            env.svm.get_account(&timed_escrow_address).unwrap(),
            timed_escrow_account_before
        );
        assert_eq!(
            env.svm.get_account(&vault_address).unwrap(),
            vault_account_before
        );
        assert_eq!(
            load::<TokenAccount>(&env, &token_fixture.maker_ata_a).amount,
            INITIAL_TOKEN_BALANCE - DEPOSIT
        );
    }
    let mut clock: Clock = env.svm.get_sysvar();
    clock.unix_timestamp = expires_at;
    env.svm.set_sysvar(&clock);
    let maker_lamports_before = env.svm.get_balance(&env.maker.pubkey()).unwrap();

    let refund_expired =
        escrow_client.refund_expired(env.maker.pubkey(), token_fixture.mint_a, SEED);

    let refund_fee = send_tx_expect_ok(&mut env.svm, refund_expired, &[&env.maker]);
    assert_eq!(
        load::<TokenAccount>(&env, &token_fixture.maker_ata_a).amount,
        INITIAL_TOKEN_BALANCE
    );
    assert!(env.svm.get_account(&timed_escrow_address).is_none());
    assert!(env.svm.get_account(&vault_address).is_none());
    assert_eq!(
        env.svm.get_balance(&env.maker.pubkey()).unwrap(),
        maker_lamports_before
            + timed_escrow_account_before.lamports
            + vault_account_before.lamports
            - refund_fee,
    );
}

#[test]
fn refund_expired_rejects_a_non_maker_after_expiry() {
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
    let mut clock: Clock = env.svm.get_sysvar();
    clock.unix_timestamp = expires_at + 1;
    env.svm.set_sysvar(&clock);

    let mut refund_expired =
        escrow_client.refund_expired(env.maker.pubkey(), token_fixture.mint_a, SEED);

    refund_expired
        .accounts
        .iter_mut()
        .find(|account| account.pubkey == env.maker.pubkey())
        .unwrap()
        .pubkey = env.other_maker.pubkey();
    refund_expired
        .accounts
        .iter_mut()
        .find(|account| account.pubkey == token_fixture.maker_ata_a)
        .unwrap()
        .pubkey = get_associated_token_address(&env.other_maker.pubkey(), &token_fixture.mint_a);
    let error = send_tx_expect_error(&mut env.svm, refund_expired, &[&env.other_maker]);
    assert!(error.contains("Error Code: UnauthorizedMaker"), "{error}");
    let (timed_escrow_address, _) = TimedEscrow::find_pda(&env.maker.pubkey(), SEED);
    let vault_address = get_associated_token_address(&timed_escrow_address, &token_fixture.mint_a);
    assert_eq!(
        load::<TimedEscrow>(&env, &timed_escrow_address).maker,
        env.maker.pubkey()
    );
    assert_eq!(load::<TokenAccount>(&env, &vault_address).amount, DEPOSIT);
}
