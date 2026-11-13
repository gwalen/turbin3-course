mod common;

use anchor_spl::{
    associated_token::{get_associated_token_address, spl_associated_token_account::instruction},
    token_interface::TokenAccount,
};
use common::{
    client::EscrowClient,
    load, send_tx_expect_error, send_tx_expect_ok,
    token::{self, TokenFixture, INITIAL_TOKEN_BALANCE},
    Env, DEPOSIT, RECEIVE, SEED,
};
use escrow::state::escrow::Escrow;
use solana_keypair::Keypair;
use solana_signer::Signer;

enum Substitution {
    Maker,
    PaymentMint,
}

#[test]
fn another_signer_cannot_refund_the_makers_escrow() {
    let mut env = Env::new();
    let token_fixture = TokenFixture::new(&mut env);
    let escrow_client = EscrowClient::new();

    let make = escrow_client.make(
        env.maker.pubkey(),
        token_fixture.mint_a,
        token_fixture.mint_b,
        SEED,
        DEPOSIT,
        RECEIVE,
    );

    send_tx_expect_ok(&mut env.svm, make, &[&env.maker]);
    let mut refund = escrow_client.refund(env.maker.pubkey(), token_fixture.mint_a, SEED);
    refund
        .accounts
        .iter_mut()
        .find(|account| account.pubkey == env.maker.pubkey())
        .unwrap()
        .pubkey = env.other_maker.pubkey();
    refund
        .accounts
        .iter_mut()
        .find(|account| account.pubkey == token_fixture.maker_ata_a)
        .unwrap()
        .pubkey = get_associated_token_address(&env.other_maker.pubkey(), &token_fixture.mint_a);
    let error = send_tx_expect_error(&mut env.svm, refund, &[&env.other_maker]);
    assert!(error.contains("Error Code: UnauthorizedMaker"), "{error}");
    let (escrow_address, _) = Escrow::find_pda(&env.maker.pubkey(), SEED);
    assert_eq!(
        load::<Escrow>(&env, &escrow_address).maker,
        env.maker.pubkey()
    );

    let vault: TokenAccount = load(
        &env,
        &get_associated_token_address(&escrow_address, &token_fixture.mint_a),
    );

    assert_eq!(vault.amount, DEPOSIT);
}

#[test]
fn take_rejects_substituted_maker_and_payment_mint() {
    for substitution in [Substitution::Maker, Substitution::PaymentMint] {
        let mut env = Env::new();
        let token_fixture = TokenFixture::new(&mut env);
        let escrow_client = EscrowClient::new();

        let make = escrow_client.make(
            env.maker.pubkey(),
            token_fixture.mint_a,
            token_fixture.mint_b,
            SEED,
            DEPOSIT,
            RECEIVE,
        );

        send_tx_expect_ok(&mut env.svm, make, &[&env.maker]);

        let mut take = escrow_client.take(
            env.maker.pubkey(),
            env.taker.pubkey(),
            token_fixture.mint_a,
            token_fixture.mint_b,
            SEED,
        );

        let expected_error = match substitution {
            Substitution::Maker => {
                take.accounts
                    .iter_mut()
                    .find(|account| account.pubkey == env.maker.pubkey())
                    .unwrap()
                    .pubkey = env.other_maker.pubkey();

                take.accounts
                    .iter_mut()
                    .find(|account| account.pubkey == token_fixture.maker_ata_b)
                    .unwrap()
                    .pubkey =
                    get_associated_token_address(&env.other_maker.pubkey(), &token_fixture.mint_b);
                "UnauthorizedMaker"
            }
            Substitution::PaymentMint => {
                let substituted_mint = Keypair::new_from_array([7; 32]);
                token::create_mint(&mut env, &substituted_mint);

                let create_taker_ata = instruction::create_associated_token_account(
                    &env.taker.pubkey(),
                    &env.taker.pubkey(),
                    &substituted_mint.pubkey(),
                    &anchor_spl::token::ID,
                );

                send_tx_expect_ok(&mut env.svm, create_taker_ata, &[&env.taker]);
                take.accounts
                    .iter_mut()
                    .find(|account| account.pubkey == token_fixture.mint_b)
                    .unwrap()
                    .pubkey = substituted_mint.pubkey();
                take.accounts
                    .iter_mut()
                    .find(|account| account.pubkey == token_fixture.taker_ata_b)
                    .unwrap()
                    .pubkey =
                    get_associated_token_address(&env.taker.pubkey(), &substituted_mint.pubkey());
                take.accounts
                    .iter_mut()
                    .find(|account| account.pubkey == token_fixture.maker_ata_b)
                    .unwrap()
                    .pubkey =
                    get_associated_token_address(&env.maker.pubkey(), &substituted_mint.pubkey());
                "InvalidMintB"
            }
        };
        let error = send_tx_expect_error(&mut env.svm, take, &[&env.taker]);
        assert!(
            error.contains(&format!("Error Code: {expected_error}")),
            "{error}"
        );
        let (escrow_address, _) = Escrow::find_pda(&env.maker.pubkey(), SEED);

        let vault: TokenAccount = load(
            &env,
            &get_associated_token_address(&escrow_address, &token_fixture.mint_a),
        );

        assert_eq!(vault.amount, DEPOSIT);
        assert_eq!(
            load::<TokenAccount>(&env, &token_fixture.taker_ata_b).amount,
            INITIAL_TOKEN_BALANCE
        );
        assert!(env.svm.get_account(&token_fixture.maker_ata_b).is_none());
    }
}
