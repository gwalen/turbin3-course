use crate::utils::errors::EscrowError;
use anchor_lang::prelude::*;
use anchor_spl::token_interface::{
    spl_token_2022::{
        extension::{BaseStateWithExtensions, ExtensionType, StateWithExtensions},
        state,
    },
    Mint,
};

pub fn check_mint_is_supported(mint: &InterfaceAccount<'_, Mint>) -> Result<()> {
    let mint_account_info = mint.to_account_info();
    let mint_data = mint_account_info.try_borrow_data()?;
    let mint_state = StateWithExtensions::<state::Mint>::unpack(&mint_data)?;
    let mint_extension_types = mint_state.get_extension_types()?;

    require!(
        !mint_extension_types.contains(&ExtensionType::TransferFeeConfig),
        EscrowError::TransferFeeMint
    );
    require!(
        !mint_extension_types.contains(&ExtensionType::PermanentDelegate),
        EscrowError::PermanentDelegateMint
    );
    require!(
        !mint_extension_types.contains(&ExtensionType::TransferHook),
        EscrowError::TransferHookMint
    );
    Ok(())
}
