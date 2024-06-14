pub use anchor_lang::prelude::*;

#[account(zero_copy)]
pub struct FeeAccount {
    pub boss: Pubkey,
    pub bump: u8,
    pub redeem_mint_bump: u8,
    pub extra_meta_bump: u8,
    pub pda_authority_bump: u8,
}

pub const FEE_ACCOUNT_SIZE: usize = 8 + 32 + 4;
