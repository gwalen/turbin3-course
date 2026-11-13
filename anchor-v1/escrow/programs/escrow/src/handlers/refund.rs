use crate::{
    instructions::Refund,
    state::escrow::Escrow,
    utils::{
        common::{self, VaultWithdrawal},
        events::EscrowRefunded,
    },
};
use anchor_lang::prelude::*;

pub fn handle(ctx: Context<Refund>) -> Result<()> {
    let amount = ctx.accounts.vault.amount;
    common::withdraw_and_close_vault(
        VaultWithdrawal {
            token_program: &ctx.accounts.token_program,
            vault: &ctx.accounts.vault,
            mint_a: &ctx.accounts.mint_a,
            destination: ctx.accounts.maker_ata_a.to_account_info(),
            maker: ctx.accounts.maker.to_account_info(),
            escrow: ctx.accounts.escrow.to_account_info(),
        },
        Escrow::SEED_PREFIX,
        ctx.accounts.escrow.seed,
        ctx.accounts.escrow.bump,
    )?;
    emit!(EscrowRefunded {
        escrow: ctx.accounts.escrow.key(),
        maker: ctx.accounts.maker.key(),
        amount,
    });
    Ok(())
}
