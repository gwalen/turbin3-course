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
fn make_stores_terms_and_locks_the_deposit() {
    let mut env = Env::new();
    let tokens = TokenFixture::new(&mut env);
    let client = EscrowClient::new();
    assert_eq!(env.program_id, escrow::ID);

    let make = client.make(
        env.maker.pubkey(),
        tokens.mint_a,
        tokens.mint_b,
        SEED,
        DEPOSIT,
        RECEIVE,
    );

    send_tx_expect_ok(&mut env.svm, make, &[&env.maker]);
    let (escrow_address, escrow_bump) = Escrow::find_pda(&env.maker.pubkey(), SEED);
    let escrow: Escrow = load(&env, &escrow_address);
    assert_eq!(escrow.seed, SEED);
    assert_eq!(escrow.maker, env.maker.pubkey());
    assert_eq!(escrow.mint_a, tokens.mint_a);
    assert_eq!(escrow.mint_b, tokens.mint_b);
    assert_eq!(escrow.receive, RECEIVE);
    assert_eq!(escrow.bump, escrow_bump);
    assert_eq!(
        env.svm.get_account(&escrow_address).unwrap().data.len(),
        Escrow::LEN
    );

    let vault: TokenAccount = load(
        &env,
        &get_associated_token_address(&escrow_address, &tokens.mint_a),
    );

    assert_eq!(vault.amount, DEPOSIT);
    assert_eq!(vault.owner, escrow_address);
    assert_eq!(vault.mint, tokens.mint_a);
    assert_eq!(
        load::<TokenAccount>(&env, &tokens.maker_ata_a).amount,
        INITIAL_TOKEN_BALANCE - DEPOSIT
    );
}

#[test]
fn make_rejects_zero_amounts_and_matching_mints() {
    for (deposit, receive, matching_mints, expected_error) in [
        (0, RECEIVE, false, "ZeroAmount"),
        (DEPOSIT, 0, false, "ZeroAmount"),
        (DEPOSIT, RECEIVE, true, "SameMint"),
    ] {
        let mut env = Env::new();
        let tokens = TokenFixture::new(&mut env);
        let client = EscrowClient::new();
        let mint_b = if matching_mints {
            tokens.mint_a
        } else {
            tokens.mint_b
        };

        let make = client.make(
            env.maker.pubkey(),
            tokens.mint_a,
            mint_b,
            SEED,
            deposit,
            receive,
        );

        let error = send_tx_expect_error(&mut env.svm, make, &[&env.maker]);
        assert!(
            error.contains(&format!("Error Code: {expected_error}")),
            "{error}"
        );
        let (escrow_address, _) = Escrow::find_pda(&env.maker.pubkey(), SEED);
        assert!(env.svm.get_account(&escrow_address).is_none());
        assert_eq!(
            load::<TokenAccount>(&env, &tokens.maker_ata_a).amount,
            INITIAL_TOKEN_BALANCE
        );
    }
}

#[test]
fn maker_can_open_independent_escrows_with_different_seeds() {
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
        let (escrow_address, _) = Escrow::find_pda(&env.maker.pubkey(), seed);
        assert_eq!(load::<Escrow>(&env, &escrow_address).seed, seed);

        let vault: TokenAccount = load(
            &env,
            &get_associated_token_address(&escrow_address, &tokens.mint_a),
        );

        assert_eq!(vault.amount, DEPOSIT);
    }
    assert_eq!(
        load::<TokenAccount>(&env, &tokens.maker_ata_a).amount,
        INITIAL_TOKEN_BALANCE - 2 * DEPOSIT
    );
}
