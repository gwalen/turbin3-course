use anchor_lang::{
    prelude::*,
    system_program::{transfer, Transfer},
};

use crate::{
    instructions::close::Close, state::vault_state::VaultState, utils::events::VaultClosed,
};

pub fn handle(ctx: Context<Close>) -> Result<()> {
    let vault_balance = ctx.accounts.vault.lamports();

    if vault_balance > 0 {
        let vault_state_key = ctx.accounts.vault_state.key();
        let vault_bump = [ctx.accounts.vault_state.vault_bump];

        let vault_signer_seeds =
            VaultState::create_vault_signer_seeds(&vault_state_key, &vault_bump);

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

        transfer(cpi_context, vault_balance)?;
    }

    emit!(VaultClosed {
        user: ctx.accounts.user.key(),
        vault: ctx.accounts.vault.key(),
        amount: vault_balance,
    });

    Ok(())
}
