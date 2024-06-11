use anchor_lang::{
    prelude::*,
    solana_program::{instruction::Instruction, program::invoke_signed},
};
use anchor_spl::{
    token_2022::spl_token_2022::{
        extension::{
            transfer_fee::TransferFeeConfig, BaseStateWithExtensions, StateWithExtensions,
        },
        state::Mint,
    },
    token_interface::TokenAccount,
};

use crate::ID;

// Order of accounts matters for this struct.
// The first 4 accounts are the accounts required for token transfer (source, mint, destination, owner)
// Remaining accounts are the extra accounts required from the ExtraAccountMetaList account
// These accounts are provided via CPI to this program from the token2022 program
#[derive(Accounts)]
pub struct TransferHookCtx<'info> {
    #[account(
        token::mint = mint,
        token::authority = owner,
    )]
    pub source_token: InterfaceAccount<'info, TokenAccount>,
    /// CHECK:
    pub mint: AccountInfo<'info>,
    #[account(
        token::mint = mint,
    )]
    pub destination_token: InterfaceAccount<'info, TokenAccount>,
    /// CHECK: source token account owner, can be SystemAccount or PDA owned by another program
    pub owner: UncheckedAccount<'info>,
    /// CHECK: ExtraAccountMetaList Account,
    #[account(
        seeds = [b"extra-account-metas", mint.key().as_ref()], 
        bump
    )]
    pub extra_account_meta_list: UncheckedAccount<'info>,
    /// CHECK: Checked by cpi
    #[account(address = ID)]
    pub program: UncheckedAccount<'info>,
    #[account(
        mut,
        seeds = [b"pda_authority", mint.key().as_ref()], 
        bump
    )]
    /// CHECK: Checked by cpi
    pub pda_authority: SystemAccount<'info>,
    /// CHECK: Checked by cpi
    pub source_fee_account: UncheckedAccount<'info>,
    /// CHECK: Checked by cpi
    pub destination_fee_account: UncheckedAccount<'info>,
    /// CHECK: Checked by cpi
    pub boss_fee_account: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

pub fn transfer_hook_handler(ctx: Context<TransferHookCtx>, amount: u64) -> Result<()> {
    let mint_info = ctx.accounts.mint.to_account_info();
    let mint_data = mint_info.data.borrow();
    let mint = StateWithExtensions::<Mint>::unpack(&mint_data)?;
    let fee = if let Ok(transfer_fee_config) = mint.get_extension::<TransferFeeConfig>() {
        let fee = transfer_fee_config
            .calculate_epoch_fee(Clock::get()?.epoch, amount)
            .ok_or(ProgramError::InvalidArgument)?;
        fee
    } else {
        0
    };

    let bump = &[ctx.bumps.pda_authority];
    let mint_key = ctx.accounts.mint.key();
    let seeds: &[&[u8]] = &[b"pda_authority".as_ref(), mint_key.as_ref(), bump];
    let signer_seeds = &[&seeds[..]];

    // cpi to self
    let mut bytes_data = Vec::with_capacity(24);
    bytes_data.extend([225, 27, 13, 6, 69, 84, 172, 191]);
    bytes_data.extend(fee.to_le_bytes());
    bytes_data.extend(amount.saturating_sub(fee).to_le_bytes());

    let account_infos: Vec<AccountInfo> = vec![
        ctx.accounts.source_token.to_account_info(),
        ctx.accounts.mint.to_account_info(),
        ctx.accounts.destination_token.to_account_info(),
        ctx.accounts.pda_authority.to_account_info(),
        ctx.accounts.source_fee_account.to_account_info(),
        ctx.accounts.destination_fee_account.to_account_info(),
        ctx.accounts.boss_fee_account.to_account_info(),
        ctx.accounts.system_program.to_account_info(),
    ];

    let accounts = vec![
        AccountMeta::new_readonly(account_infos[0].key(), false),
        AccountMeta::new_readonly(account_infos[1].key(), false),
        AccountMeta::new_readonly(account_infos[2].key(), false),
        AccountMeta::new(account_infos[3].key(), true),
        AccountMeta::new(account_infos[4].key(), false),
        AccountMeta::new(account_infos[5].key(), false),
        AccountMeta::new(account_infos[6].key(), false),
        AccountMeta::new_readonly(account_infos[7].key(), false),
    ];

    invoke_signed(
        &Instruction {
            program_id: ID,
            accounts,
            data: bytes_data,
        },
        &account_infos[..],
        signer_seeds,
    )?;

    Ok(())
}
