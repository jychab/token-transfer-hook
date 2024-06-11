use anchor_lang::prelude::*;

#[error_code]
pub enum TokenTransferHook {
    #[msg("Unauthorized Invoke")]
    Unauthorized,
}
