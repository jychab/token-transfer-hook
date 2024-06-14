use anchor_lang::{
    prelude::*,
    solana_program::{incinerator, system_program},
};
use anchor_spl::{
    associated_token::{self, AssociatedToken},
    token_interface::{
        freeze_account, mint_to, thaw_account, FreezeAccount, Mint, MintTo, ThawAccount,
        TokenAccount, TokenInterface,
    },
};

use crate::{
    state::{FeeAccount, FEE_ACCOUNT_SIZE},
    ID,
};

#[derive(Accounts)]
pub struct TransferFeesCtx<'info> {
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
        bump,
    )]
    pub destination_fee_account: AccountLoader<'info, FeeAccount>,
    /// CHECK:
    #[account(mut)]
    pub redeem_mint: AccountInfo<'info>,
    /// CHECK:
    pub boss: Option<AccountInfo<'info>>,
    /// CHECK:
    #[account(mut)]
    pub boss_redemption_token_account: AccountInfo<'info>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
}

pub fn transfer_fees_handler(
    ctx: Context<TransferFeesCtx>,
    fees: u64,
    amount_after_fee: u64,
) -> Result<()> {
    let source_fee_account = &mut ctx.accounts.source_fee_account.load_mut()?;
    if ctx.accounts.destination_token.amount == amount_after_fee {
        let destination_fee_account = &mut ctx
            .accounts
            .destination_fee_account
            .load_mut()
            .unwrap_or(ctx.accounts.destination_fee_account.load_init()?);
        destination_fee_account.boss = ctx.accounts.source_token.owner;
        destination_fee_account.bump = ctx.bumps.destination_fee_account;
        destination_fee_account.pda_authority_bump = source_fee_account.pda_authority_bump;
        destination_fee_account.extra_meta_bump = source_fee_account.extra_meta_bump;
        destination_fee_account.redeem_mint_bump = source_fee_account.redeem_mint_bump;
    }

    if ctx.accounts.source_token.amount == 0 {
        source_fee_account.boss = ID;
    }

    if let Some(boss) = &ctx.accounts.boss {
        if ctx.accounts.boss_redemption_token_account.get_lamports() == 0
            || system_program::check_id(&ctx.accounts.boss_redemption_token_account.owner)
            || incinerator::check_id(&ctx.accounts.boss_redemption_token_account.owner)
        {
            associated_token::create_idempotent(CpiContext::new(
                ctx.accounts.associated_token_program.to_account_info(),
                associated_token::Create {
                    payer: ctx.accounts.payer.to_account_info(),
                    associated_token: ctx.accounts.boss_redemption_token_account.to_account_info(),
                    authority: boss.to_account_info(),
                    mint: ctx.accounts.mint.to_account_info(),
                    system_program: ctx.accounts.system_program.to_account_info(),
                    token_program: ctx.accounts.token_program.to_account_info(),
                },
            ))?;
        } else {
            thaw_account(CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                ThawAccount {
                    account: ctx.accounts.boss_redemption_token_account.to_account_info(),
                    mint: ctx.accounts.redeem_mint.to_account_info(),
                    authority: ctx.accounts.payer.to_account_info(),
                },
            ))?;
        }

        mint_to(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    mint: ctx.accounts.redeem_mint.to_account_info(),
                    to: ctx.accounts.boss_redemption_token_account.to_account_info(),
                    authority: ctx.accounts.payer.to_account_info(),
                },
            ),
            fees,
        )?;

        freeze_account(CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            FreezeAccount {
                account: ctx.accounts.boss_redemption_token_account.to_account_info(),
                mint: ctx.accounts.redeem_mint.to_account_info(),
                authority: ctx.accounts.payer.to_account_info(),
            },
        ))?;
    }

    Ok(())
}
