use crate::{state::escrow::Escrow, utils::errors::EscrowError};
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::{get_associated_token_address_with_program_id, AssociatedToken},
    memo::Memo,
    token_interface::{Mint, TokenAccount, TokenInterface},
};

#[derive(Accounts)]
pub struct Take<'info> {
    #[account(mut)]
    pub taker: Signer<'info>,
    /// CHECK: The escrow has_one constraint binds this account to its maker.
    #[account(mut)]
    pub maker: UncheckedAccount<'info>,
    #[account(mint::token_program = token_program_a)]
    pub mint_a: InterfaceAccount<'info, Mint>,
    #[account(mint::token_program = token_program_b)]
    pub mint_b: InterfaceAccount<'info, Mint>,
    #[account(
        mut, close = maker,
        has_one = maker @ EscrowError::UnauthorizedMaker,
        has_one = mint_a @ EscrowError::InvalidMintA,
        has_one = mint_b @ EscrowError::InvalidMintB,
        seeds = [Escrow::SEED_PREFIX, escrow.maker.as_ref(), &escrow.seed.to_le_bytes()],
        bump = escrow.bump,
    )]
    pub escrow: Account<'info, Escrow>,
    #[account(
        init_if_needed, payer = taker,
        associated_token::mint = mint_a,
        associated_token::authority = taker,
        associated_token::token_program = token_program_a,
    )]
    pub taker_ata_a: InterfaceAccount<'info, TokenAccount>,
    #[account(
        mut,
        associated_token::mint = mint_b,
        associated_token::authority = taker,
        associated_token::token_program = token_program_b,
    )]
    pub taker_ata_b: InterfaceAccount<'info, TokenAccount>,
    /// CHECK: The address binds the payment ATA; the handler validates its program and mint.
    #[account(
        mut,
        address = get_associated_token_address_with_program_id(
            &maker.key(), &mint_b.key(), &token_program_b.key()
        ),
    )]
    pub maker_ata_b: UncheckedAccount<'info>,
    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = escrow,
        associated_token::token_program = token_program_a,
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,
    pub token_program_a: Interface<'info, TokenInterface>,
    pub token_program_b: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub memo_program: Program<'info, Memo>,
    pub system_program: Program<'info, System>,
}
