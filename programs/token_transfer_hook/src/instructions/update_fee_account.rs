use anchor_lang::{prelude::*, solana_program::sysvar};
use anchor_spl::token_interface::Mint;
use spl_token_2022::cmp_pubkeys;

use crate::{
    error::TokenTransferHook,
    state::{FeeAccount, FeeUpdateEvent, FEE_ACCOUNT_SIZE, TOKEN_CREATOR_PROGRAM_ID},
};

#[event_cpi]
#[derive(Accounts)]
#[instruction(address:Pubkey)]
pub struct UpdateFeeAccountCtx<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        init_if_needed,
        seeds = [b"fee", mint.key().as_ref(),address.as_ref()], 
        bump,
        payer = payer,
        space = FEE_ACCOUNT_SIZE,
    )]
    pub fee_account: AccountLoader<'info, FeeAccount>,
    /// CHECK: account constraints checked in account trait
    #[account(address = sysvar::instructions::id())]
    pub instruction_sysvar_account: UncheckedAccount<'info>,
    pub mint: InterfaceAccount<'info, Mint>,
    pub system_program: Program<'info, System>,
}

pub fn update_fee_account_handler(
    ctx: Context<UpdateFeeAccountCtx>,
    address: Pubkey,
    boss: Option<Pubkey>,
    additional_claimed_fees: u64,
    additional_unclaimed_fees: u64,
) -> Result<()> {
    // only allow this method to be invoked by the token creator program
    let instruction =
        sysvar::instructions::get_instruction_relative(0, &ctx.accounts.instruction_sysvar_account)
            .unwrap();
    require!(
        cmp_pubkeys(&instruction.program_id, &TOKEN_CREATOR_PROGRAM_ID),
        TokenTransferHook::Unauthorized
    );
    let fee_account = &mut ctx
        .accounts
        .fee_account
        .load_mut()
        .or(ctx.accounts.fee_account.load_init())?;

    if let Some(new_boss) = boss {
        fee_account.boss = new_boss;
        fee_account.unclaimed_fees = 0;
        fee_account.claimed_fees = 0
    } else {
        fee_account.claimed_fees += additional_claimed_fees;
        fee_account.unclaimed_fees += additional_unclaimed_fees;
    }

    emit_cpi!(FeeUpdateEvent {
        address: address,
        boss: fee_account.boss,
        unclaimed_fees: fee_account.unclaimed_fees,
        claimed_fees: fee_account.claimed_fees
    });
    Ok(())
}
