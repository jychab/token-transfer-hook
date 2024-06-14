use anchor_lang::prelude::*;

pub mod error;
pub mod instructions;
pub mod state;

use spl_transfer_hook_interface::instruction::TransferHookInstruction;

pub use instructions::*;

declare_id!("FNeSgS1XbVmbxZgNBHfpXxh6aivuTSf3hSCncT6QDm9T");

#[program]
pub mod token_transfer_hook {

    use super::*;

    // lamports here is used to fund the pda that will pay for the creation of PDAs during transfer hook
    pub fn initialize_extra_account_meta_list(
        ctx: Context<InitializeExtraAccountMetaListCtx>,
    ) -> Result<()> {
        instructions::initialize_extra_meta::initialize_extra_account_meta_list_handler(ctx)
    }

    pub fn create_mint(
        ctx: Context<CreateMintCtx>,
        random_key: Pubkey,
        fee_basis_pts: u16,
        max_fee: u64,
        decimals: u8,
    ) -> Result<()> {
        instructions::create_mint::create_mint_handler(
            ctx,
            random_key,
            fee_basis_pts,
            max_fee,
            decimals,
        )
    }

    pub fn redeem_mint<'info>(ctx: Context<'_, '_, '_, 'info, RedeemMintCtx<'info>>) -> Result<()> {
        instructions::redeem_mint::redeem_mint_handler(ctx)
    }

    pub fn create_mint_metadata(
        ctx: Context<CreateMintMetadataCtx>,
        lamports: u64,
        name: String,
        symbol: String,
        uri: String,
    ) -> Result<()> {
        instructions::create_mint_metadata::create_mint_metadata_handler(
            ctx, lamports, name, symbol, uri,
        )
    }

    pub fn mint_to(ctx: Context<MintCtx>, amount: u64) -> Result<()> {
        instructions::mint::mint_to_handler(ctx, amount)
    }

    // this method should only be cpi by transfer_hook
    pub fn transfer_fees(
        ctx: Context<TransferFeesCtx>,
        fees: u64,
        amount_after_fee: u64,
    ) -> Result<()> {
        instructions::transfer_fees::transfer_fees_handler(ctx, fees, amount_after_fee)
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
