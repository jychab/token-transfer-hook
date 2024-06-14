use anchor_lang::{
    prelude::*,
    solana_program::native_token::LAMPORTS_PER_SOL,
    system_program::{transfer, Transfer},
};
use anchor_spl::{
    token_2022::Token2022,
    token_interface::{
        initialize_mint, metadata_pointer_initialize, transfer_fee_initialize,
        transfer_hook_initialize, InitializeMint, MetadataPointerInitialize, Mint, TokenInterface,
        TransferFeeInitialize, TransferHookInitialize,
    },
};

use crate::ID;

#[derive(Accounts)]
#[instruction(random_key:Pubkey, decimals:u8)]
pub struct CreateMintCtx<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        init,
        payer = payer,
        seeds = [b"mint", random_key.as_ref()],
        bump,
        owner = Token2022::id(),
        space = 414,
    )]
    /// CHECK: Mint to be created
    pub mint: AccountInfo<'info>,
    #[account(
        init,
        payer = payer,
        seeds = [b"redeem", mint.key().as_ref()],
        bump,
        mint::token_program = token_program,
        mint::authority = pda_authority,
        mint::freeze_authority = pda_authority,
        mint::decimals = decimals,
    )]
    /// CHECK: Mint to be created
    pub redeem_mint: InterfaceAccount<'info, Mint>,
    #[account(
        mut,
        seeds = [b"pda_authority", mint.key().as_ref()],
        bump,
    )]
    pub pda_authority: SystemAccount<'info>,
    pub token_2022_program: Interface<'info, TokenInterface>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn create_mint_handler(
    ctx: Context<CreateMintCtx>,
    _random_key: Pubkey,
    fee_basis_pts: u16,
    max_fee: u64,
    decimals: u8,
) -> Result<()> {
    transfer(
        CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            Transfer {
                from: ctx.accounts.payer.to_account_info(),
                to: ctx.accounts.pda_authority.to_account_info(),
            },
        ),
        LAMPORTS_PER_SOL,
    )?;

    // initialize transfer fee
    transfer_fee_initialize(
        CpiContext::new(
            ctx.accounts.token_2022_program.to_account_info(),
            TransferFeeInitialize {
                token_program_id: ctx.accounts.token_2022_program.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
            },
        ),
        Some(&ctx.accounts.pda_authority.key()),
        Some(&ctx.accounts.pda_authority.key()),
        fee_basis_pts,
        max_fee,
    )?;

    // initialize transfer hook
    transfer_hook_initialize(
        CpiContext::new(
            ctx.accounts.token_2022_program.to_account_info(),
            TransferHookInitialize {
                token_program_id: ctx.accounts.token_2022_program.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
            },
        ),
        Some(ctx.accounts.pda_authority.key()),
        Some(ID),
    )?;

    // initialize mint metadata pointer
    metadata_pointer_initialize(
        CpiContext::new(
            ctx.accounts.token_2022_program.to_account_info(),
            MetadataPointerInitialize {
                token_program_id: ctx.accounts.token_2022_program.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
            },
        ),
        Some(ctx.accounts.pda_authority.key()),
        Some(ctx.accounts.mint.key()),
    )?;

    // intialize mint
    initialize_mint(
        CpiContext::new(
            ctx.accounts.token_2022_program.to_account_info(),
            InitializeMint {
                mint: ctx.accounts.mint.to_account_info(),
                rent: ctx.accounts.rent.to_account_info(),
            },
        ),
        decimals,
        &ctx.accounts.pda_authority.key(),
        None,
    )?;

    Ok(())
}
