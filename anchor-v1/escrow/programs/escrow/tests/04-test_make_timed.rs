mod common;

use anchor_spl::{associated_token::get_associated_token_address, token_interface::TokenAccount};
use common::{
    client::EscrowClient,
    env::INITIAL_TIMESTAMP,
    load, send_tx_expect_error, send_tx_expect_ok,
    token::{TokenFixture, INITIAL_TOKEN_BALANCE},
    Env, DEPOSIT, RECEIVE, SEED,
};
use escrow::state::{escrow::Escrow, timed_escrow::TimedEscrow};
use solana_signer::Signer;

#[test]
fn make_timed_stores_expiry_and_locks_tokens_in_a_distinct_vault() {
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

    let (timed_escrow_address, timed_escrow_bump) =
        TimedEscrow::find_pda(&env.maker.pubkey(), SEED);

    let timed_escrow: TimedEscrow = load(&env, &timed_escrow_address);
    assert_eq!(timed_escrow.seed, SEED);
    assert_eq!(timed_escrow.maker, env.maker.pubkey());
    assert_eq!(timed_escrow.mint_a, token_fixture.mint_a);
    assert_eq!(timed_escrow.mint_b, token_fixture.mint_b);
    assert_eq!(timed_escrow.receive, RECEIVE);
    assert_eq!(timed_escrow.expires_at, expires_at);
    assert_eq!(timed_escrow.bump, timed_escrow_bump);
    assert_eq!(
        env.svm
            .get_account(&timed_escrow_address)
            .unwrap()
            .data
            .len(),
        TimedEscrow::LEN
    );
    let vault_address = get_associated_token_address(&timed_escrow_address, &token_fixture.mint_a);
    let vault: TokenAccount = load(&env, &vault_address);
    assert_eq!(vault.amount, DEPOSIT);
    assert_eq!(vault.owner, timed_escrow_address);
    assert_eq!(vault.mint, token_fixture.mint_a);
    assert_eq!(
        load::<TokenAccount>(&env, &token_fixture.maker_ata_a).amount,
        INITIAL_TOKEN_BALANCE - DEPOSIT
    );

    let make = escrow_client.make(
        env.maker.pubkey(),
        token_fixture.mint_a,
        token_fixture.mint_b,
        SEED,
        DEPOSIT,
        RECEIVE,
    );

    send_tx_expect_ok(&mut env.svm, make, &[&env.maker]);
    let (escrow_address, _) = Escrow::find_pda(&env.maker.pubkey(), SEED);
    assert_ne!(escrow_address, timed_escrow_address);
    assert!(env.svm.get_account(&escrow_address).is_some());
    assert_eq!(load::<TokenAccount>(&env, &vault_address).amount, DEPOSIT);
}

#[test]
fn make_timed_rejects_present_and_past_expiries() {
    for expires_at in [INITIAL_TIMESTAMP, INITIAL_TIMESTAMP - 1] {
        let mut env = Env::new();
        let token_fixture = TokenFixture::new(&mut env);
        let escrow_client = EscrowClient::new();

        let make_timed = escrow_client.make_timed(
            env.maker.pubkey(),
            token_fixture.mint_a,
            token_fixture.mint_b,
            SEED,
            DEPOSIT,
            RECEIVE,
            expires_at,
        );

        let error = send_tx_expect_error(&mut env.svm, make_timed, &[&env.maker]);
        assert!(error.contains("Error Code: InvalidExpiry"), "{error}");
        let (timed_escrow_address, _) = TimedEscrow::find_pda(&env.maker.pubkey(), SEED);
        assert!(env.svm.get_account(&timed_escrow_address).is_none());

        let vault_address =
            get_associated_token_address(&timed_escrow_address, &token_fixture.mint_a);

        assert!(env.svm.get_account(&vault_address).is_none());
        assert_eq!(
            load::<TokenAccount>(&env, &token_fixture.maker_ata_a).amount,
            INITIAL_TOKEN_BALANCE
        );
    }
}
