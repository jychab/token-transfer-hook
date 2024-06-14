use anchor_lang::{prelude::*, system_program};
use anchor_spl::token_interface::{
    token_metadata_initialize, Mint, TokenInterface, TokenMetadataInitialize,
};

#[derive(Accounts)]
pub struct CreateMintMetadataCtx<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        mut,
    )]
    pub mint: InterfaceAccount<'info, Mint>,
    #[account(
        mut,
        seeds = [b"redeem", mint.key().as_ref()],
        bump 
    )]
    pub redeem_mint: InterfaceAccount<'info, Mint>,
    #[account(
        seeds = [b"pda_authority", mint.key().as_ref()],
        bump 
    )]
    pub pda_authority: SystemAccount<'info>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

pub fn create_mint_metadata_handler(
    ctx: Context<CreateMintMetadataCtx>,
    lamports: u64,
    name: String,
    symbol: String,
    uri: String,
) -> Result<()> {
    let bump = &[ctx.bumps.pda_authority];
    let mint_key = ctx.accounts.mint.key();
    let seeds: &[&[u8]] = &[b"pda_authority".as_ref(), mint_key.as_ref(), bump];
    let signer_seeds = &[&seeds[..]];

    system_program::transfer(
        CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: ctx.accounts.payer.to_account_info(),
                to: ctx.accounts.mint.to_account_info(),
            },
        ),
        lamports,
    )?;

    token_metadata_initialize(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            TokenMetadataInitialize {
                token_program_id: ctx.accounts.token_program.to_account_info(),
                metadata: ctx.accounts.mint.to_account_info(),
                update_authority: ctx.accounts.pda_authority.to_account_info(),
                mint_authority: ctx.accounts.pda_authority.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
            },
        )
        .with_signer(signer_seeds),
        name,
        symbol,
        uri,
    )?;

    

    Ok(())
}
