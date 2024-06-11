pub use anchor_lang::prelude::*;
use solana_program::pubkey;

#[account(zero_copy)]
pub struct FeeAccount {
    pub boss: Pubkey,
    pub unclaimed_fees: u64,
    pub claimed_fees: u64,
    pub bump: u8,
    pub extra_meta_bump: u8,
    pub pda_authority_bump: u8,
    pub padding: [u8; 5],
}

pub const FEE_ACCOUNT_SIZE: usize = 8 + 32 + 8 + 8 + 8;

pub const TOKEN_CREATOR_PROGRAM_ID: Pubkey =
    pubkey!("9pw2AQd1yBosbbncXzzFLpmqyWfGCZkrmY8EZeJVP7z2");

#[event]
pub struct TransferEvent {
    pub mint: Pubkey,
    pub source: Pubkey,
    pub source_boss: Pubkey,
    pub destination: Pubkey,
    pub destination_boss: Option<Pubkey>,
    pub destination_token_account: Pubkey,
    pub boss: Pubkey,
    pub boss_unclaimed_fee: u64,
}

#[event]
pub struct FeeUpdateEvent {
    pub mint: Pubkey,
    pub address: Pubkey,
    pub boss: Pubkey,
    pub unclaimed_fees: u64,
    pub claimed_fees: u64,
}
