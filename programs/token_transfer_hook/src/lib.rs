use anchor_lang::prelude::*;

pub mod error;
pub mod instructions;
pub mod state;

use spl_transfer_hook_interface::instruction::TransferHookInstruction;

pub use instructions::*;

declare_id!("HTdsvYSXbpLEvP5bzYVWEczMVK5VyKRhqRSx3E5c4VYV");

#[program]
pub mod token_transfer_hook {

    use super::*;

    // lamports here is used to fund the pda that will pay for the creation of PDAs during transfer hook
    pub fn initialize_extra_account_meta_list(
        ctx: Context<InitializeExtraAccountMetaListCtx>,
        lamports: u64,
    ) -> Result<()> {
        instructions::initialize_extra_meta::initialize_extra_account_meta_list_handler(
            ctx, lamports,
        )
    }

    // this method is only allowed to be called from the token creator program
    pub fn update_fee_account(
        ctx: Context<UpdateFeeAccountCtx>,
        address: Pubkey,
        boss: Option<Pubkey>,
        additional_claimed_fees: u64,
        additional_unclaimed_fees: u64,
    ) -> Result<()> {
        instructions::update_fee_account::update_fee_account_handler(
            ctx,
            address,
            boss,
            additional_claimed_fees,
            additional_unclaimed_fees,
        )
    }

    // this method should only be cpi by transfer_hook
    pub fn update_fees(ctx: Context<UpdateFeesCtx>, fee: u64, amount_after_fee: u64) -> Result<()> {
        instructions::update_fees::update_fees_handler(ctx, fee, amount_after_fee)
    }

    pub fn transfer_hook(ctx: Context<TransferHookCtx>, amount: u64) -> Result<()> {
        instructions::transfer_hook::transfer_hook_handler(ctx, amount)
    }

    // fallback instruction handler as workaround to anchor instruction discriminator check
    pub fn fallback<'info>(
        program_id: &Pubkey,
        accounts: &'info [AccountInfo<'info>],
        data: &[u8],
    ) -> Result<()> {
        let instruction = TransferHookInstruction::unpack(data)?;

        // match instruction discriminator to transfer hook interface execute instruction
        // token2022 program CPIs this instruction on token transfer
        match instruction {
            TransferHookInstruction::Execute { amount } => {
                let amount_bytes = amount.to_le_bytes();

                // invoke custom transfer hook instruction on our program
                __private::__global::transfer_hook(program_id, accounts, &amount_bytes)
            }
            _ => return Err(ProgramError::InvalidInstructionData.into()),
        }
    }
}
