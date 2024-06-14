use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{mint_to, Mint, MintTo, TokenAccount, TokenInterface},
};
#[derive(Accounts)]
pub struct MintCtx<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = mint,
        associated_token::authority = payer
    )]
    pub source_token_account: InterfaceAccount<'info, TokenAccount>,
    #[account(mut)]
    pub mint: InterfaceAccount<'info, Mint>,
    #[account(
        seeds = [b"pda_authority", mint.key().as_ref()],
        bump,
    )]
    pub pda_authority: SystemAccount<'info>,
    #[account(
        init_if_needed,
        payer = payer,
		associated_token::mint = mint,
		associated_token::authority = pda_authority,
        associated_token::token_program = token_program,
	)]
    pub pda_authority_mint: Box<InterfaceAccount<'info, TokenAccount>>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn mint_to_handler(ctx: Context<MintCtx>, amount: u64) -> Result<()> {
    let mint_key = ctx.accounts.mint.key();
    let seeds = &[
        b"pda_authority",
        mint_key.as_ref(),
        &[ctx.bumps.pda_authority],
    ];
    let signer = &[&seeds[..]];
    mint_to(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.mint.to_account_info(),
                to: ctx.accounts.source_token_account.to_account_info(),
                authority: ctx.accounts.pda_authority.to_account_info(),
            },
        )
        .with_signer(signer),
        amount,
    )?;

    Ok(())
}
