mod common;

use anchor_lang::solana_program::program_pack::Pack;
use anchor_spl::{
    associated_token::{
        get_associated_token_address, get_associated_token_address_with_program_id,
    },
    token::spl_token::state::Account as SplTokenAccount,
    token_2022,
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
enum MintRole {
    Deposit,
    Payment,
}

#[test]
fn make_rejects_permanent_delegate_on_either_mint() {
    assert_mint_extension_rejected(
        OfferKind::Basic,
        Token2022MintExtension::PermanentDelegate,
        "PermanentDelegateMint",
    );
}

#[test]
fn make_timed_rejects_permanent_delegate_on_either_mint() {
    assert_mint_extension_rejected(
        OfferKind::Timed,
        Token2022MintExtension::PermanentDelegate,
        "PermanentDelegateMint",
    );
}

#[test]
fn make_rejects_enabled_and_disabled_transfer_hooks_on_either_mint() {
    for program_id in [None, Some(escrow::ID)] {
        assert_mint_extension_rejected(
            OfferKind::Basic,
            Token2022MintExtension::TransferHook { program_id },
            "TransferHookMint",
        );
    }
}

#[test]
fn make_timed_rejects_enabled_and_disabled_transfer_hooks_on_either_mint() {
    for program_id in [None, Some(escrow::ID)] {
        assert_mint_extension_rejected(
            OfferKind::Timed,
            Token2022MintExtension::TransferHook { program_id },
            "TransferHookMint",
        );
    }
}

#[test]
fn take_rejects_empty_vault_before_payment() {
    assert_empty_vault_rejected(OfferKind::Basic);
}

#[test]
fn take_expired_rejects_empty_vault_before_payment() {
    assert_empty_vault_rejected(OfferKind::Timed);
}

fn assert_mint_extension_rejected(
    offer_kind: OfferKind,
    mint_extension: Token2022MintExtension,
    expected_error_code: &str,
) {
    for mint_role in [MintRole::Deposit, MintRole::Payment] {
        let mut escrow_env = Env::new();

        let (mint_a_extension, mint_b_extension) = match mint_role {
            MintRole::Deposit => (mint_extension, Token2022MintExtension::None),
            MintRole::Payment => (Token2022MintExtension::None, mint_extension),
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
            error_text.contains(&format!("Error Code: {expected_error_code}")),
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

fn assert_empty_vault_rejected(offer_kind: OfferKind) {
    let mut escrow_env = Env::new();
    let token_fixture = TokenFixture::new(&mut escrow_env);
    let escrow_client = EscrowClient::new();
    let maker = escrow_env.maker.pubkey();
    let taker = escrow_env.taker.pubkey();

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
    let vault_address = get_associated_token_address(&escrow_address, &token_fixture.mint_a);
    let mut vault_account = escrow_env.svm.get_account(&vault_address).unwrap();
    let mut vault_token_account = SplTokenAccount::unpack(&vault_account.data).unwrap();
    // This snapshot represents an offer whose vault was already emptied.
    vault_token_account.amount = 0;
    SplTokenAccount::pack(vault_token_account, &mut vault_account.data).unwrap();
    escrow_env
        .svm
        .set_account(vault_address, vault_account)
        .unwrap();
    let vault_account_before = escrow_env.svm.get_account(&vault_address).unwrap();
    let escrow_account_before = escrow_env.svm.get_account(&escrow_address).unwrap();
    let maker_lamports_before = escrow_env.svm.get_balance(&maker).unwrap();

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

    let error_text =
        send_tx_expect_error(&mut escrow_env.svm, take_instruction, &[&escrow_env.taker]);

    assert!(
        error_text.contains("Error Code: EmptyVault"),
        "{error_text}"
    );
    assert!(
        !error_text.contains("Instruction: TransferChecked"),
        "{error_text}"
    );
    assert_eq!(
        load::<TokenAccount>(&escrow_env, &token_fixture.taker_ata_b).amount,
        INITIAL_TOKEN_BALANCE
    );
    assert!(escrow_env
        .svm
        .get_account(&token_fixture.maker_ata_b)
        .is_none());
    assert!(escrow_env
        .svm
        .get_account(&token_fixture.taker_ata_a)
        .is_none());
    assert_eq!(
        escrow_env.svm.get_account(&vault_address).unwrap(),
        vault_account_before
    );
    assert_eq!(
        escrow_env.svm.get_account(&escrow_address).unwrap(),
        escrow_account_before
    );
    assert_eq!(
        escrow_env.svm.get_balance(&maker).unwrap(),
        maker_lamports_before
    );
}
