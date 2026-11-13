mod common;

use anchor_lang::prelude::Clock;
use anchor_spl::{
    associated_token::get_associated_token_address_with_program_id, token_interface::TokenAccount,
};
use common::{
    client::EscrowClient,
    env::INITIAL_TIMESTAMP,
    load, send_tx_expect_ok,
    token::{Token2022MintExtension, TokenFixture, TokenMintKind, INITIAL_TOKEN_BALANCE},
    Env, DEPOSIT, RECEIVE, SEED,
};
use escrow::state::{escrow::Escrow, timed_escrow::TimedEscrow};
use solana_signer::Signer;

#[derive(Clone, Copy)]
enum OfferKind {
    Basic,
    Timed,
}

#[test]
fn mixed_program_basic_offers_swap_and_refund_in_both_directions() {
    assert_mixed_program_lifecycle(OfferKind::Basic);
}

#[test]
fn mixed_program_timed_offers_swap_and_refund_in_both_directions() {
    assert_mixed_program_lifecycle(OfferKind::Timed);
}

fn assert_mixed_program_lifecycle(offer_kind: OfferKind) {
    for (mint_a_kind, mint_b_kind) in [
        (
            TokenMintKind::Classic,
            TokenMintKind::Token2022(Token2022MintExtension::None),
        ),
        (
            TokenMintKind::Token2022(Token2022MintExtension::None),
            TokenMintKind::Classic,
        ),
    ] {
        let mut escrow_env = Env::new();

        let token_fixture =
            TokenFixture::create_with_programs(&mut escrow_env, mint_a_kind, mint_b_kind);

        let mut escrow_client = EscrowClient::new();
        escrow_client.token_program_a = mint_a_kind.token_program_id();
        escrow_client.token_program_b = mint_b_kind.token_program_id();
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
        assert!(escrow_env
            .svm
            .get_account(&token_fixture.maker_ata_b)
            .is_none());
        assert!(escrow_env
            .svm
            .get_account(&token_fixture.taker_ata_a)
            .is_none());

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
        assert_eq!(
            escrow_env
                .svm
                .get_account(&token_fixture.maker_ata_b)
                .unwrap()
                .owner,
            escrow_client.token_program_b
        );
        assert_eq!(
            escrow_env
                .svm
                .get_account(&token_fixture.taker_ata_a)
                .unwrap()
                .owner,
            escrow_client.token_program_a
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
                &escrow_client.token_program_a,
            );

            assert!(escrow_env.svm.get_account(&escrow_address).is_none());
            assert!(escrow_env.svm.get_account(&vault_address).is_none());
        }
    }
}
