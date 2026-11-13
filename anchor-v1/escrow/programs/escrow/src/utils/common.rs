use anchor_lang::prelude::*;
use anchor_spl::token_interface::{
    self, CloseAccount, Mint, TokenAccount, TokenInterface, TransferChecked,
};

pub struct VaultWithdrawal<'a, 'info> {
    pub token_program: &'a Interface<'info, TokenInterface>,
    pub vault: &'a InterfaceAccount<'info, TokenAccount>,
    pub mint_a: &'a InterfaceAccount<'info, Mint>,
    pub destination: AccountInfo<'info>,
    pub maker: AccountInfo<'info>,
    pub escrow: AccountInfo<'info>,
}

pub fn withdraw_and_close_vault(
    withdrawal: VaultWithdrawal<'_, '_>,
    seed_prefix: &[u8],
    seed: u64,
    bump: u8,
) -> Result<()> {
    let seed_bytes = seed.to_le_bytes();
    let bump_bytes = [bump];
    let escrow_signer_seeds = [
        seed_prefix,
        withdrawal.maker.key.as_ref(),
        &seed_bytes,
        &bump_bytes,
    ];
    let signer_seeds = &[escrow_signer_seeds.as_slice()];

    token_interface::transfer_checked(
        CpiContext::new(
            withdrawal.token_program.key(),
            TransferChecked {
                from: withdrawal.vault.to_account_info(),
                mint: withdrawal.mint_a.to_account_info(),
                to: withdrawal.destination,
                authority: withdrawal.escrow.clone(),
            },
        )
        .with_signer(signer_seeds),
        withdrawal.vault.amount,
        withdrawal.mint_a.decimals,
    )?;
    token_interface::close_account(
        CpiContext::new(
            withdrawal.token_program.key(),
            CloseAccount {
                account: withdrawal.vault.to_account_info(),
                destination: withdrawal.maker,
                authority: withdrawal.escrow,
            },
        )
        .with_signer(signer_seeds),
    )
}
