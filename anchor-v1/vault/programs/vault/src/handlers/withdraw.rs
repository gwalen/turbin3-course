use anchor_lang::{
    prelude::*,
    system_program::{transfer, Transfer},
};

use crate::{
    instructions::withdraw::Withdraw, state::vault_state::VaultState, utils::errors::VaultError,
    utils::events::Withdrawn,
};

pub fn handle(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
    require!(amount > 0, VaultError::ZeroAmount);

    let vault_lamports = ctx.accounts.vault.lamports();

    let remaining = vault_lamports
        .checked_sub(amount)
        .ok_or(VaultError::InsufficientFunds)?;

    require!(
        remaining == 0 || remaining >= VaultState::MIN_BALANCE,
        VaultError::BelowMinBalance
    );

    let vault_state_key = ctx.accounts.vault_state.key();
    let vault_bump = [ctx.accounts.vault_state.vault_bump];

    let vault_signer_seeds = VaultState::create_vault_signer_seeds(&vault_state_key, &vault_bump);

    let signer_seeds: &[&[u8]] = &vault_signer_seeds;

    let cpi_accounts = Transfer {
        from: ctx.accounts.vault.to_account_info(),
        to: ctx.accounts.user.to_account_info(),
    };

    let signer_seed_refs = [signer_seeds];

    let cpi_context = CpiContext::new_with_signer(
        ctx.accounts.system_program.key(),
        cpi_accounts,
        &signer_seed_refs,
    );

    transfer(cpi_context, amount)?;

    emit!(Withdrawn {
        user: ctx.accounts.user.key(),
        vault: ctx.accounts.vault.key(),
        amount,
        vault_balance: remaining,
    });

    Ok(())
}
