mod common;

use anchor_lang::{prelude::Pubkey, solana_program::system_instruction};
use anchor_spl::{
    associated_token::{
        get_associated_token_address_with_program_id,
        spl_associated_token_account::instruction::create_associated_token_account,
    },
    token::{
        self,
        spl_token::instruction::{set_authority, AuthorityType},
    },
    token_2022::{
        self,
        spl_token_2022::{
            extension::{
                memo_transfer::instruction::enable_required_transfer_memos, ExtensionType,
            },
            instruction::reallocate,
        },
    },
    token_interface::TokenAccount,
};
use common::{
    client::EscrowClient,
    env::INITIAL_TIMESTAMP,
    load, send_tx_expect_ok,
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

#[test]
fn take_succeeds_after_maker_assigns_wallet_to_another_program() {
    assert_reassigned_maker_can_receive_settlement(OfferKind::Basic);
}

#[test]
fn take_expired_succeeds_after_maker_assigns_wallet_to_another_program() {
    assert_reassigned_maker_can_receive_settlement(OfferKind::Timed);
}

#[test]
fn take_pays_a_memo_required_token_2022_account() {
    assert_memo_required_payment_succeeds(OfferKind::Basic);
}

#[test]
fn take_expired_pays_a_memo_required_token_2022_account() {
    assert_memo_required_payment_succeeds(OfferKind::Timed);
}

#[test]
fn take_pays_after_maker_reassigns_payment_account_authority() {
    assert_reassigned_payment_account_receives_settlement(OfferKind::Basic);
}

#[test]
fn take_expired_pays_after_maker_reassigns_payment_account_authority() {
    assert_reassigned_payment_account_receives_settlement(OfferKind::Timed);
}

fn assert_reassigned_payment_account_receives_settlement(offer_kind: OfferKind) {
    let mut escrow_env = Env::new();
    let token_fixture = TokenFixture::new(&mut escrow_env);
    let escrow_client = EscrowClient::new();
    let escrow_address = create_offer(&mut escrow_env, &token_fixture, &escrow_client, offer_kind);
    let maker = escrow_env.maker.pubkey();
    let new_payment_account_authority = escrow_env.taker.pubkey();

    let create_payment_account =
        create_associated_token_account(&maker, &maker, &token_fixture.mint_b, &token::ID);

    send_tx_expect_ok(
        &mut escrow_env.svm,
        create_payment_account,
        &[&escrow_env.maker],
    );

    let reassign_payment_account = set_authority(
        &token::ID,
        &token_fixture.maker_ata_b,
        Some(&new_payment_account_authority),
        AuthorityType::AccountOwner,
        &maker,
        &[],
    )
    .unwrap();

    send_tx_expect_ok(
        &mut escrow_env.svm,
        reassign_payment_account,
        &[&escrow_env.maker],
    );
    take_offer(&mut escrow_env, &token_fixture, &escrow_client, offer_kind);

    let vault_address = get_associated_token_address_with_program_id(
        &escrow_address,
        &token_fixture.mint_a,
        &token::ID,
    );

    assert_eq!(
        load::<TokenAccount>(&escrow_env, &token_fixture.maker_ata_b).owner,
        new_payment_account_authority
    );
    assert_swap_completed(&escrow_env, &token_fixture, &escrow_address, &vault_address);
}

fn assert_reassigned_maker_can_receive_settlement(offer_kind: OfferKind) {
    let mut escrow_env = Env::new();
    let token_fixture = TokenFixture::new(&mut escrow_env);
    let escrow_client = EscrowClient::new();
    let escrow_address = create_offer(&mut escrow_env, &token_fixture, &escrow_client, offer_kind);
    let maker = escrow_env.maker.pubkey();

    let vault_address = get_associated_token_address_with_program_id(
        &escrow_address,
        &token_fixture.mint_a,
        &escrow_client.token_program_a,
    );

    let closed_account_rent = escrow_env.svm.get_balance(&escrow_address).unwrap()
        + escrow_env.svm.get_balance(&vault_address).unwrap();
    let assign_maker = system_instruction::assign(&maker, &escrow::ID);
    send_tx_expect_ok(&mut escrow_env.svm, assign_maker, &[&escrow_env.maker]);
    let maker_lamports_before = escrow_env.svm.get_balance(&maker).unwrap();
    take_offer(&mut escrow_env, &token_fixture, &escrow_client, offer_kind);
    assert_eq!(
        escrow_env.svm.get_account(&maker).unwrap().owner,
        escrow::ID
    );
    assert_eq!(
        escrow_env.svm.get_balance(&maker).unwrap(),
        maker_lamports_before + closed_account_rent
    );
    assert_swap_completed(&escrow_env, &token_fixture, &escrow_address, &vault_address);
}

fn assert_memo_required_payment_succeeds(offer_kind: OfferKind) {
    let mut escrow_env = Env::new();

    let token_fixture = TokenFixture::create_token_2022(
        &mut escrow_env,
        Token2022MintExtension::None,
        Token2022MintExtension::None,
    );

    let mut escrow_client = EscrowClient::new();
    escrow_client.token_program_a = token_2022::ID;
    escrow_client.token_program_b = token_2022::ID;
    let escrow_address = create_offer(&mut escrow_env, &token_fixture, &escrow_client, offer_kind);
    let maker = escrow_env.maker.pubkey();

    let create_payment_account =
        create_associated_token_account(&maker, &maker, &token_fixture.mint_b, &token_2022::ID);

    send_tx_expect_ok(
        &mut escrow_env.svm,
        create_payment_account,
        &[&escrow_env.maker],
    );

    let reallocate_payment_account = reallocate(
        &token_2022::ID,
        &token_fixture.maker_ata_b,
        &maker,
        &maker,
        &[],
        &[ExtensionType::MemoTransfer],
    )
    .unwrap();

    send_tx_expect_ok(
        &mut escrow_env.svm,
        reallocate_payment_account,
        &[&escrow_env.maker],
    );

    let enable_memos =
        enable_required_transfer_memos(&token_2022::ID, &token_fixture.maker_ata_b, &maker, &[])
            .unwrap();

    send_tx_expect_ok(&mut escrow_env.svm, enable_memos, &[&escrow_env.maker]);
    take_offer(&mut escrow_env, &token_fixture, &escrow_client, offer_kind);

    let vault_address = get_associated_token_address_with_program_id(
        &escrow_address,
        &token_fixture.mint_a,
        &token_2022::ID,
    );

    assert_swap_completed(&escrow_env, &token_fixture, &escrow_address, &vault_address);
}

fn create_offer(
    escrow_env: &mut Env,
    token_fixture: &TokenFixture,
    escrow_client: &EscrowClient,
    offer_kind: OfferKind,
) -> Pubkey {
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

    send_tx_expect_ok(&mut escrow_env.svm, make_instruction, &[&escrow_env.maker]);
    escrow_address
}

fn take_offer(
    escrow_env: &mut Env,
    token_fixture: &TokenFixture,
    escrow_client: &EscrowClient,
    offer_kind: OfferKind,
) {
    let maker = escrow_env.maker.pubkey();
    let taker = escrow_env.taker.pubkey();

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
}

fn assert_swap_completed(
    escrow_env: &Env,
    token_fixture: &TokenFixture,
    escrow_address: &Pubkey,
    vault_address: &Pubkey,
) {
    assert_eq!(
        load::<TokenAccount>(escrow_env, &token_fixture.maker_ata_b).amount,
        RECEIVE
    );
    assert_eq!(
        load::<TokenAccount>(escrow_env, &token_fixture.taker_ata_a).amount,
        DEPOSIT
    );
    assert_eq!(
        load::<TokenAccount>(escrow_env, &token_fixture.taker_ata_b).amount,
        INITIAL_TOKEN_BALANCE - RECEIVE
    );
    assert!(escrow_env.svm.get_account(escrow_address).is_none());
    assert!(escrow_env.svm.get_account(vault_address).is_none());
}
