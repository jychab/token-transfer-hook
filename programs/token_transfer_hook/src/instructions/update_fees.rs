use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount};

use crate::state::{FeeAccount, FEE_ACCOUNT_SIZE, TOKEN_CREATOR_PROGRAM_ID};

#[derive(Accounts)]
pub struct UpdateFeesCtx<'info> {
    pub source_token: InterfaceAccount<'info, TokenAccount>,
    pub mint: InterfaceAccount<'info, Mint>,
    pub destination_token: InterfaceAccount<'info, TokenAccount>,
    #[account(
        mut,
        seeds = [b"pda_authority", mint.key().as_ref()], 
        bump,
    )]
    pub payer: Signer<'info>,
    #[account(
        mut,
        seeds = [b"fee", mint.key().as_ref(), source_token.owner.as_ref()],
        bump
    )]
    pub source_fee_account: AccountLoader<'info, FeeAccount>,
    #[account(
        init_if_needed,
        payer = payer,
        space = FEE_ACCOUNT_SIZE,
        seeds = [b"fee", mint.key().as_ref(), destination_token.owner.as_ref()],
        bump
    )]
    pub destination_fee_account: AccountLoader<'info, FeeAccount>,
    #[account(
        init_if_needed,
        payer = payer,
        space = FEE_ACCOUNT_SIZE,
        seeds = [b"fee", mint.key().as_ref(), source_fee_account.load()?.boss.as_ref()],
        bump
    )]
    pub boss_fee_account: AccountLoader<'info, FeeAccount>,
    pub system_program: Program<'info, System>,
}

pub fn update_fees_handler(
    ctx: Context<UpdateFeesCtx>,
    fee: u64,
    amount_after_fee: u64,
) -> Result<()> {
    if ctx.accounts.destination_token.amount == amount_after_fee {
        let destination_fee_account = &mut ctx
            .accounts
            .destination_fee_account
            .load_mut()
            .or(ctx.accounts.destination_fee_account.load_init())?;
        destination_fee_account.boss = ctx.accounts.source_token.owner;
    }
    if ctx.accounts.source_token.amount == 0 {
        let source_fee_account = &mut ctx.accounts.source_fee_account.load_mut()?;
        source_fee_account.boss = TOKEN_CREATOR_PROGRAM_ID;
    }
    let boss_fee_account = &mut ctx
        .accounts
        .boss_fee_account
        .load_mut()
        .or(ctx.accounts.boss_fee_account.load_init())?;
    boss_fee_account.unclaimed_fees += fee;
    Ok(())
}
