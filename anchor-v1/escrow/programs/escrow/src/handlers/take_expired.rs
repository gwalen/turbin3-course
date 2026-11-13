use crate::{
    instructions::TakeExpired,
    state::timed_escrow::TimedEscrow,
    utils::{
        common::{self, VaultWithdrawal},
        errors::EscrowError,
        events::EscrowTaken,
        payment,
    },
};
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::Create,
    memo::{self, BuildMemo},
    token_2022,
    token_interface::{self, TransferChecked},
};

pub fn handle(ctx: Context<TakeExpired>) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    require!(now < ctx.accounts.escrow.expires_at, EscrowError::Expired);
    let vault_amount = ctx.accounts.vault.amount;
    require!(vault_amount > 0, EscrowError::EmptyVault);
    payment::prepare_maker_payment_account(CpiContext::new(
        ctx.accounts.associated_token_program.key(),
        Create {
            payer: ctx.accounts.taker.to_account_info(),
            associated_token: ctx.accounts.maker_ata_b.to_account_info(),
            authority: ctx.accounts.maker.to_account_info(),
            mint: ctx.accounts.mint_b.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
            token_program: ctx.accounts.token_program_b.to_account_info(),
        },
    ))?;
    if ctx.accounts.token_program_b.key() == token_2022::ID {
        memo::build_memo(
            CpiContext::new(ctx.accounts.memo_program.key(), BuildMemo {}),
            b"Escrow payment",
        )?;
    }
    token_interface::transfer_checked(
        CpiContext::new(
            ctx.accounts.token_program_b.key(),
            TransferChecked {
                from: ctx.accounts.taker_ata_b.to_account_info(),
                mint: ctx.accounts.mint_b.to_account_info(),
                to: ctx.accounts.maker_ata_b.to_account_info(),
                authority: ctx.accounts.taker.to_account_info(),
            },
        ),
        ctx.accounts.escrow.receive,
        ctx.accounts.mint_b.decimals,
    )?;
    common::withdraw_and_close_vault(
        VaultWithdrawal {
            token_program: &ctx.accounts.token_program_a,
            vault: &ctx.accounts.vault,
            mint_a: &ctx.accounts.mint_a,
            destination: ctx.accounts.taker_ata_a.to_account_info(),
            maker: ctx.accounts.maker.to_account_info(),
            escrow: ctx.accounts.escrow.to_account_info(),
        },
        TimedEscrow::SEED_PREFIX,
        ctx.accounts.escrow.seed,
        ctx.accounts.escrow.bump,
    )?;
    emit!(EscrowTaken {
        escrow: ctx.accounts.escrow.key(),
        maker: ctx.accounts.maker.key(),
        taker: ctx.accounts.taker.key(),
        deposit: vault_amount,
        receive: ctx.accounts.escrow.receive,
    });
    Ok(())
}
