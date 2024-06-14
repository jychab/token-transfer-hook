use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{
        burn, harvest_withheld_tokens_to_mint, thaw_account, withdraw_withheld_tokens_from_mint,
        Burn, HarvestWithheldTokensToMint, Mint, ThawAccount, TokenAccount, TokenInterface,
        WithdrawWithheldTokensFromMint,
    },
};
use spl_token_2022::onchain::invoke_transfer_checked;

#[derive(Accounts)]
pub struct RedeemMintCtx<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    /// CHECK: Mint to be created
    #[account(mut)]
    pub mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(
		mut,
        seeds = [b"redeem", mint.key().as_ref()],
        bump,
    )]
    /// CHECK: Mint to be created
    pub redeem_mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        seeds = [b"pda_authority", mint.key().as_ref()],
        bump,
    )]
    pub pda_authority: SystemAccount<'info>,
    #[account(
		init_if_needed,
		payer = payer,
		associated_token::mint = redeem_mint,
		associated_token::authority = payer,
        associated_token::token_program = token_program,
	)]
    pub payer_redeem_mint: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
		token::mint = mint,
		token::authority = payer,
        token::token_program = token_2022_program,
	)]
    pub payer_mint: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
		token::mint = mint,
		token::authority = pda_authority,
        token::token_program = token_2022_program,
	)]
    pub pda_authority_mint: Box<InterfaceAccount<'info, TokenAccount>>,
    pub token_2022_program: Interface<'info, TokenInterface>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn redeem_mint_handler<'info>(
    ctx: Context<'_, '_, '_, 'info, RedeemMintCtx<'info>>,
) -> Result<()> {
    let amount = ctx.accounts.payer_redeem_mint.amount;
    if amount == 0 {
        return Ok(());
    }

    let bump = &[ctx.bumps.pda_authority];
    let mint_key = ctx.accounts.mint.key();
    let seeds: &[&[u8]] = &[b"pda_authority".as_ref(), mint_key.as_ref(), bump];
    let signer_seeds = &[&seeds[..]];
    thaw_account(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            ThawAccount {
                account: ctx.accounts.payer_redeem_mint.to_account_info(),
                mint: ctx.accounts.redeem_mint.to_account_info(),
                authority: ctx.accounts.pda_authority.to_account_info(),
            },
        )
        .with_signer(signer_seeds),
    )?;
    burn(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Burn {
                mint: ctx.accounts.redeem_mint.to_account_info(),
                from: ctx.accounts.payer_redeem_mint.to_account_info(),
                authority: ctx.accounts.pda_authority.to_account_info(),
            },
        )
        .with_signer(signer_seeds),
        amount,
    )?;

    harvest_withheld_tokens_to_mint(
        CpiContext::new(
            ctx.accounts.token_2022_program.to_account_info(),
            HarvestWithheldTokensToMint {
                token_program_id: ctx.accounts.token_2022_program.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
            },
        ),
        vec![
            ctx.accounts.payer_mint.to_account_info(),
            ctx.accounts.pda_authority_mint.to_account_info(),
        ],
    )?;

    withdraw_withheld_tokens_from_mint(
        CpiContext::new(
            ctx.accounts.token_2022_program.to_account_info(),
            WithdrawWithheldTokensFromMint {
                token_program_id: ctx.accounts.token_2022_program.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
                destination: ctx.accounts.pda_authority_mint.to_account_info(),
                authority: ctx.accounts.pda_authority.to_account_info(),
            },
        )
        .with_signer(signer_seeds),
    )?;

    invoke_transfer_checked(
        ctx.accounts.token_2022_program.key,
        ctx.accounts.pda_authority_mint.to_account_info(),
        ctx.accounts.mint.to_account_info(),
        ctx.accounts.payer_mint.to_account_info(),
        ctx.accounts.pda_authority.to_account_info(),
        ctx.remaining_accounts,
        amount,
        ctx.accounts.mint.decimals,
        signer_seeds,
    )?;

    Ok(())
}
