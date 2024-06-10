use anchor_lang::prelude::*;

pub mod instructions;

use spl_transfer_hook_interface::instruction::TransferHookInstruction;

pub use instructions::*;

declare_id!("HTdsvYSXbpLEvP5bzYVWEczMVK5VyKRhqRSx3E5c4VYV");

#[program]
pub mod token_transfer_hook {

    use super::*;

    pub fn initialize_extra_account_meta_list(
        ctx: Context<InitializeExtraAccountMetaListCtx>,
    ) -> Result<()> {
        instructions::initialize_extra_meta::initialize_extra_account_meta_list_handler(ctx)
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
