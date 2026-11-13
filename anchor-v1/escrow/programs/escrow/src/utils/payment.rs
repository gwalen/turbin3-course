use anchor_lang::{error::ErrorCode, prelude::*};
use anchor_spl::{
    associated_token::{self, Create},
    token_interface::TokenAccount,
};

pub fn prepare_maker_payment_account<'info>(
    ctx: CpiContext<'_, '_, '_, 'info, Create<'info>>,
) -> Result<()> {
    if ctx.accounts.associated_token.data_is_empty() {
        associated_token::create(ctx)
    } else {
        require_keys_eq!(
            *ctx.accounts.associated_token.owner,
            ctx.accounts.token_program.key(),
            ErrorCode::AccountOwnedByWrongProgram
        );
        let maker_ata_b_data = ctx.accounts.associated_token.try_borrow_data()?;
        let maker_ata_b = TokenAccount::try_deserialize(&mut maker_ata_b_data.as_ref())?;
        require_keys_eq!(
            maker_ata_b.mint,
            ctx.accounts.mint.key(),
            ErrorCode::ConstraintTokenMint
        );
        Ok(())
    }
}
