use anchor_lang::{
    prelude::*,
    system_program::{create_account, CreateAccount},
};
use anchor_spl::{associated_token::AssociatedToken, token::Token, token_interface::Mint};
use spl_tlv_account_resolution::{
    account::ExtraAccountMeta, seeds::Seed, state::ExtraAccountMetaList,
};
use spl_transfer_hook_interface::instruction::ExecuteInstruction;

use crate::{
    state::{FeeAccount, FEE_ACCOUNT_SIZE},
    ID,
};

#[derive(Accounts)]
pub struct InitializeExtraAccountMetaListCtx<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    /// CHECK: ExtraAccountMetaList Account, must use these seeds
    #[account(
        mut,
        seeds = [b"extra-account-metas", mint.key().as_ref()], 
        bump
    )]
    pub extra_account_meta_list: AccountInfo<'info>,
    pub mint: InterfaceAccount<'info, Mint>,
    #[account(
        seeds = [b"redeem", mint.key().as_ref()],
        bump
    )]
    pub redeem_mint: InterfaceAccount<'info, Mint>,
    /// CHECK: Checked by Cpi
    #[account(
        init,
        payer = payer,
        space = FEE_ACCOUNT_SIZE,
        seeds = [b"fee", mint.key().as_ref(), payer.key().as_ref()],
        bump,
    )]
    pub payer_fee_account: AccountLoader<'info, FeeAccount>,
    /// CHECK: Checked by cpi
    #[account(
        init,
        payer = payer,
        space = FEE_ACCOUNT_SIZE,
        seeds = [b"fee", mint.key().as_ref(), pda_authority.key().as_ref()],
        bump,
    )]
    pub pda_fee_account: AccountLoader<'info, FeeAccount>,
    #[account(
        seeds = [b"pda_authority", mint.key().as_ref()],
        bump,
    )]
    pub pda_authority: SystemAccount<'info>,
    pub system_program: Program<'info, System>,
}

pub fn initialize_extra_account_meta_list_handler(
    ctx: Context<InitializeExtraAccountMetaListCtx>,
) -> Result<()> {
    let payer_fee_account = &mut ctx.accounts.payer_fee_account.load_init()?;
    payer_fee_account.boss = ID;
    payer_fee_account.bump = ctx.bumps.payer_fee_account;
    payer_fee_account.pda_authority_bump = ctx.bumps.pda_authority;
    payer_fee_account.extra_meta_bump = ctx.bumps.extra_account_meta_list;
    payer_fee_account.redeem_mint_bump = ctx.bumps.redeem_mint;

    let pda_fee_account = &mut ctx.accounts.pda_fee_account.load_init()?;
    pda_fee_account.boss = ID;
    pda_fee_account.bump = ctx.bumps.payer_fee_account;
    pda_fee_account.pda_authority_bump = ctx.bumps.pda_authority;
    pda_fee_account.extra_meta_bump = ctx.bumps.extra_account_meta_list;
    pda_fee_account.redeem_mint_bump = ctx.bumps.redeem_mint;

    // The `addExtraAccountsToInstruction` JS helper function resolving incorrectly
    let account_metas = vec![
        ExtraAccountMeta::new_with_pubkey(&AssociatedToken::id(), false, false)?,
        ExtraAccountMeta::new_with_pubkey(&Token::id(), false, false)?,
        ExtraAccountMeta::new_with_pubkey(&ctx.accounts.system_program.key, false, false)?,
        ExtraAccountMeta::new_with_seeds(
            &[
                Seed::Literal {
                    bytes: "pda_authority".as_bytes().to_vec(),
                },
                Seed::AccountKey { index: 1 },
            ],
            false, // is_signer
            true,  // is_writable
        )?,
        ExtraAccountMeta::new_with_seeds(
            &[
                Seed::Literal {
                    bytes: "fee".as_bytes().to_vec(),
                },
                Seed::AccountKey { index: 1 },
                Seed::AccountData {
                    account_index: 0,
                    data_index: 32,
                    length: 32,
                },
            ],
            false, // is_signer
            true,  // is_writable
        )?,
        ExtraAccountMeta::new_with_seeds(
            &[
                Seed::Literal {
                    bytes: "fee".as_bytes().to_vec(),
                },
                Seed::AccountKey { index: 1 },
                Seed::AccountData {
                    account_index: 2,
                    data_index: 32,
                    length: 32,
                },
            ],
            false, // is_signer
            true,  // is_writable
        )?,
        ExtraAccountMeta::new_with_seeds(
            &[
                Seed::Literal {
                    bytes: "redeem".as_bytes().to_vec(),
                },
                Seed::AccountKey { index: 1 },
            ],
            false, // is_signer
            true,  // is_writable
        )?,
        ExtraAccountMeta::new_with_pubkey(&ID, false, false)?, // todo: blocked by spl libary
        ExtraAccountMeta::new_external_pda_with_seeds(
            5,
            &[
                Seed::AccountData {
                    account_index: 9,
                    data_index: 8,
                    length: 32,
                },
                Seed::AccountKey { index: 6 },
                Seed::AccountKey { index: 11 },
            ],
            false, // is_signer
            true,  // is_writable
        )?,
        ExtraAccountMeta::new_with_pubkey(&ID, false, false)?,
    ];

    // calculate account size
    let account_size = ExtraAccountMetaList::size_of(account_metas.len())? as u64;
    // calculate minimum required lamports
    let lamports = Rent::get()?.minimum_balance(account_size as usize);

    let mint = ctx.accounts.mint.key();
    let signer_seeds: &[&[&[u8]]] = &[&[
        b"extra-account-metas",
        &mint.as_ref(),
        &[ctx.bumps.extra_account_meta_list],
    ]];

    // create ExtraAccountMetaList account
    create_account(
        CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            CreateAccount {
                from: ctx.accounts.payer.to_account_info(),
                to: ctx.accounts.extra_account_meta_list.to_account_info(),
            },
        )
        .with_signer(signer_seeds),
        lamports,
        account_size,
        ctx.program_id,
    )?;

    // initialize ExtraAccountMetaList account with extra accounts
    ExtraAccountMetaList::init::<ExecuteInstruction>(
        &mut ctx.accounts.extra_account_meta_list.try_borrow_mut_data()?,
        &account_metas,
    )?;

    Ok(())
}
