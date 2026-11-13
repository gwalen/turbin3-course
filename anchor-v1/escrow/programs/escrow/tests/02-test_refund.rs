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
fn refund_returns_tokens_and_rents_and_prevents_a_later_take() {
    let mut env = Env::new();
    let tokens = TokenFixture::new(&mut env);
    let client = EscrowClient::new();

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
    let refund = client.refund(env.maker.pubkey(), tokens.mint_a, SEED);
    let refund_fee = send_tx_expect_ok(&mut env.svm, refund, &[&env.maker]);
    assert_eq!(
        load::<TokenAccount>(&env, &tokens.maker_ata_a).amount,
        INITIAL_TOKEN_BALANCE
    );
    assert!(env.svm.get_account(&escrow_address).is_none());
    assert!(env.svm.get_account(&vault_address).is_none());
    assert_eq!(
        env.svm.get_balance(&env.maker.pubkey()).unwrap(),
        maker_lamports_before + escrow_rent + vault_rent - refund_fee
    );

    let take = client.take(
        env.maker.pubkey(),
        env.taker.pubkey(),
        tokens.mint_a,
        tokens.mint_b,
        SEED,
    );

    let error = send_tx_expect_error(&mut env.svm, take, &[&env.taker]);
    assert!(
        error.contains("Error Code: AccountNotInitialized"),
        "{error}"
    );
}

#[test]
fn refund_after_take_fails_because_the_offer_is_closed() {
    let mut env = Env::new();
    let tokens = TokenFixture::new(&mut env);
    let client = EscrowClient::new();

    let make = client.make(
        env.maker.pubkey(),
        tokens.mint_a,
        tokens.mint_b,
        SEED,
        DEPOSIT,
        RECEIVE,
    );

    send_tx_expect_ok(&mut env.svm, make, &[&env.maker]);

    let take = client.take(
        env.maker.pubkey(),
        env.taker.pubkey(),
        tokens.mint_a,
        tokens.mint_b,
        SEED,
    );

    send_tx_expect_ok(&mut env.svm, take, &[&env.taker]);
    let refund = client.refund(env.maker.pubkey(), tokens.mint_a, SEED);
    let error = send_tx_expect_error(&mut env.svm, refund, &[&env.maker]);
    assert!(
        error.contains("Error Code: AccountNotInitialized"),
        "{error}"
    );
    assert_eq!(
        load::<TokenAccount>(&env, &tokens.taker_ata_a).amount,
        DEPOSIT
    );
}

#[test]
fn refund_recreates_the_makers_closed_token_account() {
    use anchor_spl::token::{
        self,
        spl_token::instruction::{burn, close_account},
    };
    let mut env = Env::new();
    let tokens = TokenFixture::new(&mut env);
    let client = EscrowClient::new();

    let make = client.make(
        env.maker.pubkey(),
        tokens.mint_a,
        tokens.mint_b,
        SEED,
        DEPOSIT,
        RECEIVE,
    );

    send_tx_expect_ok(&mut env.svm, make, &[&env.maker]);

    let burn_remaining = burn(
        &token::ID,
        &tokens.maker_ata_a,
        &tokens.mint_a,
        &env.maker.pubkey(),
        &[],
        INITIAL_TOKEN_BALANCE - DEPOSIT,
    )
    .unwrap();

    send_tx_expect_ok(&mut env.svm, burn_remaining, &[&env.maker]);

    let close_maker_ata = close_account(
        &token::ID,
        &tokens.maker_ata_a,
        &env.maker.pubkey(),
        &env.maker.pubkey(),
        &[],
    )
    .unwrap();

    send_tx_expect_ok(&mut env.svm, close_maker_ata, &[&env.maker]);
    assert!(env.svm.get_account(&tokens.maker_ata_a).is_none());
    let refund = client.refund(env.maker.pubkey(), tokens.mint_a, SEED);
    send_tx_expect_ok(&mut env.svm, refund, &[&env.maker]);
    assert_eq!(
        load::<TokenAccount>(&env, &tokens.maker_ata_a).amount,
        DEPOSIT
    );
}
