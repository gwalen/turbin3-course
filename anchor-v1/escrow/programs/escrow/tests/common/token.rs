use super::{send_tx_expect_ok, Env};
use anchor_lang::{
    prelude::Pubkey,
    solana_program::{program_pack::Pack, system_instruction},
};
use anchor_spl::{
    associated_token::{
        get_associated_token_address_with_program_id,
        spl_associated_token_account::instruction::create_associated_token_account,
    },
    token::{
        self,
        spl_token::{instruction::initialize_mint2, state::Mint},
    },
    token_2022::{
        self,
        spl_token_2022::{
            self,
            extension::{
                transfer_fee::instruction::initialize_transfer_fee_config,
                transfer_hook::instruction as transfer_hook_instruction, ExtensionType,
            },
            instruction,
        },
    },
};
use solana_keypair::Keypair;
use solana_signer::Signer;

pub const TOKEN_DECIMALS: u8 = 6;
pub const INITIAL_TOKEN_BALANCE: u64 = 1_000_000;

#[derive(Clone, Copy)]
pub enum Token2022MintExtension {
    None,
    TransferFee { basis_points: u16 },
    PermanentDelegate,
    TransferHook { program_id: Option<Pubkey> },
}

pub struct TokenFixture {
    pub mint_a: Pubkey,
    pub mint_b: Pubkey,
    pub maker_ata_a: Pubkey,
    pub maker_ata_b: Pubkey,
    pub taker_ata_a: Pubkey,
    pub taker_ata_b: Pubkey,
}

#[derive(Clone, Copy)]
pub enum TokenMintKind {
    Classic,
    Token2022(Token2022MintExtension),
}

impl TokenMintKind {
    pub fn token_program_id(self) -> Pubkey {
        match self {
            Self::Classic => token::ID,
            Self::Token2022(_) => token_2022::ID,
        }
    }
}

impl TokenFixture {
    pub fn create_token_2022(
        escrow_env: &mut Env,
        mint_a_extension: Token2022MintExtension,
        mint_b_extension: Token2022MintExtension,
    ) -> Self {
        Self::create_with_programs(
            escrow_env,
            TokenMintKind::Token2022(mint_a_extension),
            TokenMintKind::Token2022(mint_b_extension),
        )
    }

    pub fn new(escrow_env: &mut Env) -> Self {
        Self::create_with_programs(escrow_env, TokenMintKind::Classic, TokenMintKind::Classic)
    }

    pub fn create_with_programs(
        escrow_env: &mut Env,
        mint_a_kind: TokenMintKind,
        mint_b_kind: TokenMintKind,
    ) -> Self {
        let mint_a_keypair = Keypair::new_from_array([4; 32]);
        let mint_b_keypair = Keypair::new_from_array([5; 32]);
        let maker = escrow_env.maker.pubkey();
        let taker = escrow_env.taker.pubkey();
        for (mint_keypair, mint_kind, token_owner) in [
            (&mint_a_keypair, mint_a_kind, maker),
            (&mint_b_keypair, mint_b_kind, taker),
        ] {
            match mint_kind {
                TokenMintKind::Classic => create_mint(escrow_env, mint_keypair),
                TokenMintKind::Token2022(mint_extension) => {
                    create_token_2022_mint(escrow_env, mint_keypair, mint_extension);
                }
            }
            let token_program_id = mint_kind.token_program_id();

            let create_ata = create_associated_token_account(
                &maker,
                &token_owner,
                &mint_keypair.pubkey(),
                &token_program_id,
            );

            send_tx_expect_ok(&mut escrow_env.svm, create_ata, &[&escrow_env.maker]);

            let token_account_address = get_associated_token_address_with_program_id(
                &token_owner,
                &mint_keypair.pubkey(),
                &token_program_id,
            );

            let mint_tokens = instruction::mint_to(
                &token_program_id,
                &mint_keypair.pubkey(),
                &token_account_address,
                &maker,
                &[],
                INITIAL_TOKEN_BALANCE,
            )
            .unwrap();

            send_tx_expect_ok(&mut escrow_env.svm, mint_tokens, &[&escrow_env.maker]);
        }
        Self {
            mint_a: mint_a_keypair.pubkey(),
            mint_b: mint_b_keypair.pubkey(),
            maker_ata_a: get_associated_token_address_with_program_id(
                &maker,
                &mint_a_keypair.pubkey(),
                &mint_a_kind.token_program_id(),
            ),
            maker_ata_b: get_associated_token_address_with_program_id(
                &maker,
                &mint_b_keypair.pubkey(),
                &mint_b_kind.token_program_id(),
            ),
            taker_ata_a: get_associated_token_address_with_program_id(
                &taker,
                &mint_a_keypair.pubkey(),
                &mint_a_kind.token_program_id(),
            ),
            taker_ata_b: get_associated_token_address_with_program_id(
                &taker,
                &mint_b_keypair.pubkey(),
                &mint_b_kind.token_program_id(),
            ),
        }
    }
}

pub fn create_mint(env: &mut Env, mint_keypair: &Keypair) {
    let maker = env.maker.pubkey();

    let create_mint = system_instruction::create_account(
        &maker,
        &mint_keypair.pubkey(),
        env.svm.minimum_balance_for_rent_exemption(Mint::LEN),
        Mint::LEN as u64,
        &token::ID,
    );

    send_tx_expect_ok(&mut env.svm, create_mint, &[&env.maker, mint_keypair]);

    let initialize_mint = initialize_mint2(
        &token::ID,
        &mint_keypair.pubkey(),
        &maker,
        None,
        TOKEN_DECIMALS,
    )
    .unwrap();

    send_tx_expect_ok(&mut env.svm, initialize_mint, &[&env.maker]);
}

fn create_token_2022_mint(
    escrow_env: &mut Env,
    mint_keypair: &Keypair,
    mint_extension: Token2022MintExtension,
) {
    let maker = escrow_env.maker.pubkey();

    let mint_extensions = match mint_extension {
        Token2022MintExtension::None => vec![],
        Token2022MintExtension::TransferFee { .. } => vec![ExtensionType::TransferFeeConfig],
        Token2022MintExtension::PermanentDelegate => vec![ExtensionType::PermanentDelegate],
        Token2022MintExtension::TransferHook { .. } => vec![ExtensionType::TransferHook],
    };

    let mint_account_len =
        ExtensionType::try_calculate_account_len::<spl_token_2022::state::Mint>(&mint_extensions)
            .unwrap();

    let create_mint_account = system_instruction::create_account(
        &maker,
        &mint_keypair.pubkey(),
        escrow_env
            .svm
            .minimum_balance_for_rent_exemption(mint_account_len),
        mint_account_len as u64,
        &token_2022::ID,
    );

    send_tx_expect_ok(
        &mut escrow_env.svm,
        create_mint_account,
        &[&escrow_env.maker, mint_keypair],
    );

    let initialize_mint_extension = match mint_extension {
        Token2022MintExtension::None => None,
        Token2022MintExtension::TransferFee { basis_points } => Some(
            initialize_transfer_fee_config(
                &token_2022::ID,
                &mint_keypair.pubkey(),
                Some(&maker),
                Some(&maker),
                basis_points,
                u64::MAX,
            )
            .unwrap(),
        ),
        Token2022MintExtension::PermanentDelegate => Some(
            instruction::initialize_permanent_delegate(
                &token_2022::ID,
                &mint_keypair.pubkey(),
                &maker,
            )
            .unwrap(),
        ),
        Token2022MintExtension::TransferHook { program_id } => Some(
            transfer_hook_instruction::initialize(
                &token_2022::ID,
                &mint_keypair.pubkey(),
                Some(maker),
                program_id,
            )
            .unwrap(),
        ),
    };

    if let Some(initialize_mint_extension) = initialize_mint_extension {
        send_tx_expect_ok(
            &mut escrow_env.svm,
            initialize_mint_extension,
            &[&escrow_env.maker],
        );
    }

    let initialize_mint = instruction::initialize_mint2(
        &token_2022::ID,
        &mint_keypair.pubkey(),
        &maker,
        None,
        TOKEN_DECIMALS,
    )
    .unwrap();

    send_tx_expect_ok(&mut escrow_env.svm, initialize_mint, &[&escrow_env.maker]);
}
