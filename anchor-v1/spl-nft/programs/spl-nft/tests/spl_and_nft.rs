//! LiteSVM coverage for the course tasks:
//! 1. Mint and transfer an SPL token.
//! 2. Mint an NFT with MPL Core.
//! 3. Update the NFT name and metadata URI as the update authority.
//! 4. Transfer NFT ownership between wallets.
//! 5. Burn the NFT and reclaim its rent.

#[path = "common.rs"]
mod common;

use {
    common::{
        load_asset, send, token_account, AIRDROP_LAMPORTS, DECIMALS, MINT_AMOUNT,
        MPL_CORE_CREATE_FEE, NFT_NAME, NFT_NAME_UPDATED, NFT_URI, NFT_URI_UPDATED, TRANSFER_AMOUNT,
    },
    litesvm::LiteSVM,
    mpl_core::{
        accounts::BaseAssetV1,
        instructions::{BurnV1Builder, CreateV1Builder, TransferV1Builder, UpdateV1Builder},
        types::UpdateAuthority,
        ID as MPL_CORE_ID,
    },
    solana_keypair::Keypair,
    solana_program_pack::Pack,
    solana_signer::Signer,
    solana_system_interface::{instruction::create_account, program::ID as SYSTEM_PROGRAM_ID},
    spl_associated_token_account_interface::{
        address::get_associated_token_address, instruction::create_associated_token_account,
    },
    spl_token_interface::{
        instruction::{initialize_mint2, mint_to, transfer},
        state::Mint,
        ID as TOKEN_PROGRAM_ID,
    },
};

#[test]
fn mint_and_transfer_spl_token() {
    let mut svm = LiteSVM::new();
    let authority = Keypair::new();
    let recipient = Keypair::new();
    let mint = Keypair::new();

    svm.airdrop(&authority.pubkey(), AIRDROP_LAMPORTS).unwrap();
    svm.airdrop(&recipient.pubkey(), AIRDROP_LAMPORTS).unwrap();

    let mint_rent = svm.minimum_balance_for_rent_exemption(Mint::LEN);
    let create_mint = create_account(
        &authority.pubkey(),
        &mint.pubkey(),
        mint_rent,
        Mint::LEN as u64,
        &TOKEN_PROGRAM_ID,
    );
    let initialize_mint = initialize_mint2(
        &TOKEN_PROGRAM_ID,
        &mint.pubkey(),
        &authority.pubkey(),
        None,
        DECIMALS,
    )
    .unwrap();
    send(
        &mut svm,
        &authority,
        &[create_mint, initialize_mint],
        &[&authority, &mint],
    );

    let mint_account = svm.get_account(&mint.pubkey()).unwrap();
    let mint_state = Mint::unpack(&mint_account.data).unwrap();
    assert_eq!(mint_state.decimals, DECIMALS);
    assert_eq!(mint_state.supply, 0);
    assert_eq!(mint_state.mint_authority, Some(authority.pubkey()).into());

    let authority_ata = get_associated_token_address(&authority.pubkey(), &mint.pubkey());
    let recipient_ata = get_associated_token_address(&recipient.pubkey(), &mint.pubkey());
    let create_authority_ata = create_associated_token_account(
        &authority.pubkey(),
        &authority.pubkey(),
        &mint.pubkey(),
        &TOKEN_PROGRAM_ID,
    );
    let create_recipient_ata = create_associated_token_account(
        &authority.pubkey(),
        &recipient.pubkey(),
        &mint.pubkey(),
        &TOKEN_PROGRAM_ID,
    );
    let mint_to_authority = mint_to(
        &TOKEN_PROGRAM_ID,
        &mint.pubkey(),
        &authority_ata,
        &authority.pubkey(),
        &[],
        MINT_AMOUNT,
    )
    .unwrap();
    send(
        &mut svm,
        &authority,
        &[
            create_authority_ata,
            create_recipient_ata,
            mint_to_authority,
        ],
        &[&authority],
    );

    assert_eq!(token_account(&svm, &authority_ata).amount, MINT_AMOUNT);
    assert_eq!(token_account(&svm, &recipient_ata).amount, 0);
    assert_eq!(
        Mint::unpack(&svm.get_account(&mint.pubkey()).unwrap().data)
            .unwrap()
            .supply,
        MINT_AMOUNT
    );

    let transfer_tokens = transfer(
        &TOKEN_PROGRAM_ID,
        &authority_ata,
        &recipient_ata,
        &authority.pubkey(),
        &[],
        TRANSFER_AMOUNT,
    )
    .unwrap();
    send(&mut svm, &authority, &[transfer_tokens], &[&authority]);

    let authority_token = token_account(&svm, &authority_ata);
    let recipient_token = token_account(&svm, &recipient_ata);
    assert_eq!(authority_token.owner, authority.pubkey());
    assert_eq!(recipient_token.owner, recipient.pubkey());
    assert_eq!(authority_token.mint, mint.pubkey());
    assert_eq!(recipient_token.mint, mint.pubkey());
    assert_eq!(authority_token.amount, MINT_AMOUNT - TRANSFER_AMOUNT);
    assert_eq!(recipient_token.amount, TRANSFER_AMOUNT);
}

#[test]
fn mint_update_transfer_and_burn_mpl_core_nft() {
    let mut svm = LiteSVM::new();
    svm.add_program(MPL_CORE_ID, include_bytes!("fixtures/mpl_core.so"))
        .unwrap();

    let update_authority = Keypair::new();
    let recipient = Keypair::new();
    let asset = Keypair::new();

    svm.airdrop(&update_authority.pubkey(), AIRDROP_LAMPORTS)
        .unwrap();
    svm.airdrop(&recipient.pubkey(), AIRDROP_LAMPORTS).unwrap();

    let create_nft = CreateV1Builder::new()
        .asset(asset.pubkey())
        .payer(update_authority.pubkey())
        .authority(Some(update_authority.pubkey()))
        .owner(Some(update_authority.pubkey()))
        .update_authority(Some(update_authority.pubkey()))
        .system_program(SYSTEM_PROGRAM_ID)
        .name(NFT_NAME.to_string())
        .uri(NFT_URI.to_string())
        .instruction();
    send(
        &mut svm,
        &update_authority,
        &[create_nft],
        &[&update_authority, &asset],
    );

    let created = load_asset(&svm, &asset.pubkey());
    assert_eq!(created.name, NFT_NAME);
    assert_eq!(created.uri, NFT_URI);
    assert_eq!(created.owner, update_authority.pubkey());
    assert_eq!(
        created.update_authority,
        UpdateAuthority::Address(update_authority.pubkey())
    );

    let update_nft = UpdateV1Builder::new()
        .asset(asset.pubkey())
        .payer(update_authority.pubkey())
        .authority(Some(update_authority.pubkey()))
        .system_program(SYSTEM_PROGRAM_ID)
        .new_name(NFT_NAME_UPDATED.to_string())
        .new_uri(NFT_URI_UPDATED.to_string())
        .instruction();
    send(
        &mut svm,
        &update_authority,
        &[update_nft],
        &[&update_authority],
    );

    let updated = load_asset(&svm, &asset.pubkey());
    assert_eq!(updated.name, NFT_NAME_UPDATED);
    assert_eq!(updated.uri, NFT_URI_UPDATED);
    assert_eq!(updated.owner, update_authority.pubkey());
    assert_eq!(
        updated.update_authority,
        UpdateAuthority::Address(update_authority.pubkey())
    );

    let transfer_nft = TransferV1Builder::new()
        .asset(asset.pubkey())
        .payer(update_authority.pubkey())
        .authority(Some(update_authority.pubkey()))
        .new_owner(recipient.pubkey())
        .system_program(Some(SYSTEM_PROGRAM_ID))
        .instruction();
    send(
        &mut svm,
        &update_authority,
        &[transfer_nft],
        &[&update_authority],
    );

    let transferred = load_asset(&svm, &asset.pubkey());
    assert_eq!(transferred.owner, recipient.pubkey());
    assert_eq!(
        transferred.update_authority,
        UpdateAuthority::Address(update_authority.pubkey())
    );
    assert_eq!(transferred.name, NFT_NAME_UPDATED);

    let rent_lamports = svm.get_account(&asset.pubkey()).unwrap().lamports;
    let payer_before = svm.get_balance(&update_authority.pubkey()).unwrap();
    let burn_nft = BurnV1Builder::new()
        .asset(asset.pubkey())
        .payer(update_authority.pubkey())
        .authority(Some(recipient.pubkey()))
        .system_program(Some(SYSTEM_PROGRAM_ID))
        .instruction();
    let burned = send(
        &mut svm,
        &update_authority,
        &[burn_nft],
        &[&update_authority, &recipient],
    );

    // Burn clears the asset to a 1-byte uninitialized account and refunds the
    // rent above that byte. The protocol create fee stays on the account.
    let tombstone_rent = svm.minimum_balance_for_rent_exemption(1);
    let closed = svm.get_account(&asset.pubkey()).unwrap();
    assert_eq!(closed.owner, MPL_CORE_ID);
    assert_eq!(closed.data, vec![0]);
    assert_eq!(closed.lamports, tombstone_rent + MPL_CORE_CREATE_FEE);
    assert!(BaseAssetV1::from_bytes(&closed.data).is_err());

    let payer_after = svm.get_balance(&update_authority.pubkey()).unwrap();
    let reclaimed = rent_lamports - closed.lamports;
    assert!(reclaimed > burned.fee);
    assert_eq!(payer_after, payer_before + reclaimed - burned.fee);
}
