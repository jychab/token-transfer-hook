use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount};

use crate::state::{FeeAccount, TransferEvent, FEE_ACCOUNT_SIZE, TOKEN_CREATOR_PROGRAM_ID};

#[event_cpi]
#[derive(Accounts)]
pub struct UpdateFeesCtx<'info> {
    pub source_token: InterfaceAccount<'info, TokenAccount>,
    pub mint: InterfaceAccount<'info, Mint>,
    pub destination_token: InterfaceAccount<'info, TokenAccount>,
    #[account(
        mut,
        seeds = [b"pda_authority", mint.key().as_ref()],
        bump = source_fee_account.load()?.pda_authority_bump,
    )]
    pub payer: Signer<'info>,
    #[account(
        mut,
        seeds = [b"fee", mint.key().as_ref(), source_token.owner.as_ref()],
        bump = source_fee_account.load()?.bump,
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
        mut,
        seeds = [b"fee", mint.key().as_ref(), source_fee_account.load()?.boss.as_ref()],
        bump = boss_fee_account.load()?.bump
    )]
    pub boss_fee_account: AccountLoader<'info, FeeAccount>,
    pub system_program: Program<'info, System>,
}

pub fn update_fees_handler(
    ctx: Context<UpdateFeesCtx>,
    fee: u64,
    amount_after_fee: u64,
) -> Result<()> {
    let source_fee_account = &mut ctx.accounts.source_fee_account.load_mut()?;
    let current_boss = source_fee_account.boss;
    let mut destination_boss = None;
    if ctx.accounts.destination_token.amount == amount_after_fee {
        let destination_fee_account = &mut ctx
            .accounts
            .destination_fee_account
            .load_mut()
            .or(ctx.accounts.destination_fee_account.load_init())?;
        destination_fee_account.boss = ctx.accounts.source_token.owner;
        destination_fee_account.extra_meta_bump = source_fee_account.extra_meta_bump;
        destination_fee_account.pda_authority_bump = source_fee_account.pda_authority_bump;
        destination_fee_account.bump = ctx.bumps.destination_fee_account;
        destination_boss = Some(destination_fee_account.boss);
    }
    if ctx.accounts.source_token.amount == 0 {
        source_fee_account.boss = TOKEN_CREATOR_PROGRAM_ID;
    }
    let boss_fee_account = &mut ctx
        .accounts
        .boss_fee_account
        .load_mut()
        .or(ctx.accounts.boss_fee_account.load_init())?;
    boss_fee_account.unclaimed_fees += fee;

    emit_cpi!(TransferEvent {
        mint: ctx.accounts.mint.key(),
        source: ctx.accounts.source_token.owner,
        source_boss: source_fee_account.boss,
        destination: ctx.accounts.destination_token.owner,
        destination_boss: destination_boss,
        destination_token_account: ctx.accounts.destination_token.key(),
        boss: current_boss,
        boss_unclaimed_fee: boss_fee_account.unclaimed_fees,
    });
    Ok(())
}
