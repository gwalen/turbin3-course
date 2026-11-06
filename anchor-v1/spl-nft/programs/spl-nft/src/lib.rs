use anchor_lang::prelude::*;

declare_id!("5iHRXXcFeA4EUnerB5FNdXHy2Tk1qmLSKZpNrvtawpPS");

#[program]
pub mod spl_nft {
    use super::*;

    pub fn initialize(_ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
