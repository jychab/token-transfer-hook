use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, 
    token_interface::{Mint, TokenAccount, TokenInterface}}
;
use spl_transfer_hook_interface::instruction::TransferHookInstruction;
use anchor_lang::system_program::{create_account, CreateAccount};
use spl_tlv_account_resolution::state::ExtraAccountMetaList;
use spl_transfer_hook_interface::instruction::ExecuteInstruction;
use spl_tlv_account_resolution::{account::ExtraAccountMeta, seeds::Seed};


declare_id!("DRmZQ8udrLAwQVPvwSeSrUYmd2SKxKT4CRuraLzAsqZQ");

#[program]
pub mod token {

    use super::*;

    pub fn initialize_extra_account_meta_list(
        ctx: Context<InitializeExtraAccountMetaList>,
    ) -> Result<()> {
        // The `addExtraAccountsToInstruction` JS helper function resolving incorrectly
        let account_metas = vec![
            ExtraAccountMeta::new_with_seeds(
                &[Seed::Literal {
                    bytes: "__event_authority".as_bytes().to_vec(),
                }],
                false, // is_signer
                false,  // is_writable
            )?,
            ExtraAccountMeta::new_with_pubkey(&ctx.program_id, false, false)?,
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
            ).with_signer(signer_seeds),
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
    
    pub fn transfer_hook(ctx: Context<TransferHook>, amount: u64) -> Result<()> {
        let time = Clock::get().unwrap().unix_timestamp;
        let mut destination = None;
        let tax_amount = amount.checked_div(10000).and_then(|f| f.checked_mul(5)).unwrap();
        if ctx.accounts.destination_token.amount == amount.checked_sub(tax_amount).unwrap() {
            destination = Some(ctx.accounts.destination_token.to_account_info().owner.key());
        }
        emit_cpi!(Event {
            reset: ctx.accounts.source_token.amount == 0,
            source: ctx.accounts.owner.key(),
            destination: destination,
            destination_token_account: ctx.accounts.destination_token.key(),
            mint: ctx.accounts.mint.key(),
            amount: tax_amount,
            time: time,
        });

        Ok(())
    }

    // fallback instruction handler as workaround to anchor instruction discriminator check
    pub fn fallback<'info>(
        program_id: &Pubkey,
        accounts: &'info [AccountInfo<'info>],
        data: &[u8],
    ) -> Result<()> {
        let instruction = TransferHookInstruction::unpack(data)?;

        // match instruction discriminator to transfer hook interface execute instruction
        // token2022 program CPIs this instruction on token transfer
        match instruction {
            TransferHookInstruction::Execute { amount } => {
                let amount_bytes = amount.to_le_bytes();

                // invoke custom transfer hook instruction on our program
                __private::__global::transfer_hook(program_id, accounts, &amount_bytes)
            }
            _ => return Err(ProgramError::InvalidInstructionData.into()),
        }
    }
}

#[derive(Accounts)]
pub struct InitializeExtraAccountMetaList<'info> {
    #[account(mut)]
    payer: Signer<'info>,

    /// CHECK: ExtraAccountMetaList Account, must use these seeds
    #[account(
        mut,
        seeds = [b"extra-account-metas", mint.key().as_ref()], 
        bump
    )]
    pub extra_account_meta_list: AccountInfo<'info>,
    pub mint: InterfaceAccount<'info, Mint>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

// Order of accounts matters for this struct.
// The first 4 accounts are the accounts required for token transfer (source, mint, destination, owner)
// Remaining accounts are the extra accounts required from the ExtraAccountMetaList account
// These accounts are provided via CPI to this program from the token2022 program
#[derive(Accounts)]
pub struct TransferHook<'info> {
    #[account(
        token::mint = mint, 
        token::authority = owner,
    )]
    pub source_token: InterfaceAccount<'info, TokenAccount>,
    pub mint: InterfaceAccount<'info, Mint>,
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
    /// CHECK: 
    #[account(seeds = [b"__event_authority"], bump)]
    pub event_authority: AccountInfo<'info>,
    /// CHECK: Self-CPI will fail if the program is not the current program
    pub program: AccountInfo<'info>,
}

#[event]
pub struct Event {
    reset: bool,
    mint:Pubkey,
    destination: Option<Pubkey>,
    destination_token_account: Pubkey,
    time: i64,
    amount: u64,
    source: Pubkey,
}

