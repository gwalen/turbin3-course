mod common;

use anchor_lang::prelude::Clock;
use anchor_spl::{
    associated_token::get_associated_token_address_with_program_id, token_2022,
    token_interface::TokenAccount,
};
use common::{
    client::EscrowClient,
    env::INITIAL_TIMESTAMP,
    load, send_tx_expect_error, send_tx_expect_ok,
    token::{Token2022MintExtension, TokenFixture, INITIAL_TOKEN_BALANCE},
    Env, DEPOSIT, RECEIVE, SEED,
};
use escrow::state::{escrow::Escrow, timed_escrow::TimedEscrow};
use solana_signer::Signer;

#[derive(Clone, Copy)]
enum OfferKind {
    Basic,
    Timed,
}

#[derive(Clone, Copy)]
enum FeeMint {
    Deposit,
    Payment,
}

#[test]
fn make_rejects_transfer_fee_deposit_mints() {
    assert_transfer_fee_mints_rejected(OfferKind::Basic, FeeMint::Deposit);
}

#[test]
fn make_timed_rejects_transfer_fee_deposit_mints() {
    assert_transfer_fee_mints_rejected(OfferKind::Timed, FeeMint::Deposit);
}

#[test]
fn make_rejects_transfer_fee_payment_mints() {
    assert_transfer_fee_mints_rejected(OfferKind::Basic, FeeMint::Payment);
}

#[test]
fn make_timed_rejects_transfer_fee_payment_mints() {
    assert_transfer_fee_mints_rejected(OfferKind::Timed, FeeMint::Payment);
}

#[test]
fn fee_free_token_2022_basic_offers_swap_and_refund() {
    assert_fee_free_token_2022_lifecycle(OfferKind::Basic);
}

#[test]
fn fee_free_token_2022_timed_offers_swap_and_refund() {
    assert_fee_free_token_2022_lifecycle(OfferKind::Timed);
}

fn assert_transfer_fee_mints_rejected(offer_kind: OfferKind, fee_mint: FeeMint) {
    for transfer_fee_basis_points in [100, 0] {
        let mut escrow_env = Env::new();

        let transfer_fee_extension = Token2022MintExtension::TransferFee {
            basis_points: transfer_fee_basis_points,
        };

        let (mint_a_extension, mint_b_extension) = match fee_mint {
            FeeMint::Deposit => (transfer_fee_extension, Token2022MintExtension::None),
            FeeMint::Payment => (Token2022MintExtension::None, transfer_fee_extension),
        };

        let token_fixture =
            TokenFixture::create_token_2022(&mut escrow_env, mint_a_extension, mint_b_extension);

        let mut escrow_client = EscrowClient::new();
        escrow_client.token_program_a = token_2022::ID;
        escrow_client.token_program_b = token_2022::ID;
        let maker = escrow_env.maker.pubkey();

        let (make_instruction, escrow_address) = match offer_kind {
            OfferKind::Basic => (
                escrow_client.make(
                    maker,
                    token_fixture.mint_a,
                    token_fixture.mint_b,
                    SEED,
                    DEPOSIT,
                    RECEIVE,
                ),
                Escrow::find_pda(&maker, SEED).0,
            ),
            OfferKind::Timed => (
                escrow_client.make_timed(
                    maker,
                    token_fixture.mint_a,
                    token_fixture.mint_b,
                    SEED,
                    DEPOSIT,
                    RECEIVE,
                    INITIAL_TIMESTAMP + 100,
                ),
                TimedEscrow::find_pda(&maker, SEED).0,
            ),
        };

        let error_text =
            send_tx_expect_error(&mut escrow_env.svm, make_instruction, &[&escrow_env.maker]);
        assert!(
            error_text.contains("Error Code: TransferFeeMint"),
            "{error_text}"
        );
        assert!(escrow_env.svm.get_account(&escrow_address).is_none());

        let vault_address = get_associated_token_address_with_program_id(
            &escrow_address,
            &token_fixture.mint_a,
            &token_2022::ID,
        );

        assert!(escrow_env.svm.get_account(&vault_address).is_none());
        assert_eq!(
            load::<TokenAccount>(&escrow_env, &token_fixture.maker_ata_a).amount,
            INITIAL_TOKEN_BALANCE
        );
        assert_eq!(
            load::<TokenAccount>(&escrow_env, &token_fixture.taker_ata_b).amount,
            INITIAL_TOKEN_BALANCE
        );
    }
}

fn assert_fee_free_token_2022_lifecycle(offer_kind: OfferKind) {
    let mut escrow_env = Env::new();
    let token_fixture = TokenFixture::create_token_2022(
        &mut escrow_env,
        Token2022MintExtension::None,
        Token2022MintExtension::None,
    );
    let mut escrow_client = EscrowClient::new();
    escrow_client.token_program_a = token_2022::ID;
    escrow_client.token_program_b = token_2022::ID;
    let maker = escrow_env.maker.pubkey();
    let taker = escrow_env.taker.pubkey();
    let expires_at = INITIAL_TIMESTAMP + 100;
    for seed in [SEED, SEED + 1] {
        let make_instruction = match offer_kind {
            OfferKind::Basic => escrow_client.make(
                maker,
                token_fixture.mint_a,
                token_fixture.mint_b,
                seed,
                DEPOSIT,
                RECEIVE,
            ),
            OfferKind::Timed => escrow_client.make_timed(
                maker,
                token_fixture.mint_a,
                token_fixture.mint_b,
                seed,
                DEPOSIT,
                RECEIVE,
                expires_at,
            ),
        };

        send_tx_expect_ok(&mut escrow_env.svm, make_instruction, &[&escrow_env.maker]);
    }

    let take_instruction = match offer_kind {
        OfferKind::Basic => escrow_client.take(
            maker,
            taker,
            token_fixture.mint_a,
            token_fixture.mint_b,
            SEED,
        ),
        OfferKind::Timed => escrow_client.take_expired(
            maker,
            taker,
            token_fixture.mint_a,
            token_fixture.mint_b,
            SEED,
        ),
    };

    send_tx_expect_ok(&mut escrow_env.svm, take_instruction, &[&escrow_env.taker]);
    assert_eq!(
        load::<TokenAccount>(&escrow_env, &token_fixture.maker_ata_b).amount,
        RECEIVE
    );
    assert_eq!(
        load::<TokenAccount>(&escrow_env, &token_fixture.taker_ata_a).amount,
        DEPOSIT
    );
    assert_eq!(
        load::<TokenAccount>(&escrow_env, &token_fixture.taker_ata_b).amount,
        INITIAL_TOKEN_BALANCE - RECEIVE
    );
    let mut clock: Clock = escrow_env.svm.get_sysvar();
    clock.unix_timestamp = expires_at;
    escrow_env.svm.set_sysvar(&clock);

    let refund_instruction = match offer_kind {
        OfferKind::Basic => escrow_client.refund(maker, token_fixture.mint_a, SEED + 1),
        OfferKind::Timed => escrow_client.refund_expired(maker, token_fixture.mint_a, SEED + 1),
    };

    send_tx_expect_ok(
        &mut escrow_env.svm,
        refund_instruction,
        &[&escrow_env.maker],
    );
    assert_eq!(
        load::<TokenAccount>(&escrow_env, &token_fixture.maker_ata_a).amount,
        INITIAL_TOKEN_BALANCE - DEPOSIT
    );
    for seed in [SEED, SEED + 1] {
        let escrow_address = match offer_kind {
            OfferKind::Basic => Escrow::find_pda(&maker, seed).0,
            OfferKind::Timed => TimedEscrow::find_pda(&maker, seed).0,
        };

        let vault_address = get_associated_token_address_with_program_id(
            &escrow_address,
            &token_fixture.mint_a,
            &token_2022::ID,
        );

        assert!(escrow_env.svm.get_account(&escrow_address).is_none());
        assert!(escrow_env.svm.get_account(&vault_address).is_none());
    }
}
