use anchor_lang::prelude::*;

mod utils;

declare_id!("B7pJLjbKUJhgdDmadvDSjjRpdFV59mg3uJTohoW3hxe2");

#[program]
pub mod pda_payers {
    use super::*;

    pub fn create_fee_vault(ctx: Context<CreateFeeVault>, amount: u64) -> Result<()> {
        (*ctx.accounts.fee_vault).bump = *ctx.bumps.get("fee_vault").unwrap();
        (*ctx.accounts.fee_vault).wallet_bump = *ctx.bumps.get("fee_vault_wallet").unwrap();

        solana_program::program::invoke(
            &solana_program::system_instruction::transfer(
                &ctx.accounts.authority.key(),
                &ctx.accounts.fee_vault_wallet.key(),
                amount,
            ),
            &[
                ctx.accounts.authority.to_account_info().clone(),
                ctx.accounts.fee_vault_wallet.to_account_info().clone(),
            ],
        )?;

        Ok(())
    }

    pub fn deposit_in_fee_vault(ctx: Context<DepositInFeeVault>, amount: u64) -> Result<()> {
        solana_program::program::invoke(
            &solana_program::system_instruction::transfer(
                &ctx.accounts.authority.key(),
                &ctx.accounts.fee_vault_wallet.key(),
                amount,
            ),
            &[
                ctx.accounts.authority.to_account_info().clone(),
                ctx.accounts.fee_vault_wallet.to_account_info().clone(),
                ctx.accounts.system_program.to_account_info().clone(),
            ],
        )?;

        Ok(())
    }

    pub fn withdraw_from_fee_vault(ctx: Context<WithdrawFromFeeVault>, amount: u64) -> Result<()> {
        let seeds = &[
            b"fee_vault_wallet".as_ref(),
            ctx.accounts.fee_vault.to_account_info().key.as_ref(),
            &[ctx.accounts.fee_vault.wallet_bump],
        ];

        solana_program::program::invoke_signed(
            &solana_program::system_instruction::transfer(
                &ctx.accounts.fee_vault_wallet.key(),
                &ctx.accounts.authority.key(),
                amount,
            ),
            &[
                ctx.accounts.fee_vault_wallet.to_account_info().clone(),
                ctx.accounts.authority.to_account_info().clone(),
                ctx.accounts.system_program.to_account_info().clone(),
            ],
            &[&seeds[..]],
        )?;

        Ok(())
    }

    pub fn create_collaborator(ctx: Context<CreateCollaborator>) -> Result<()> {
        let fee_vault_wallet_seeds = &[
            b"fee_vault_wallet".as_ref(),
            ctx.accounts.fee_vault.to_account_info().key.as_ref(),
            &[ctx.accounts.fee_vault.wallet_bump],
        ];
        let collaborator_seeds = &[
            b"collaborator".as_ref(),
            ctx.accounts.fee_vault.to_account_info().key.as_ref(),
            ctx.accounts
                .collaborator_base
                .to_account_info()
                .key
                .as_ref(),
            &[*ctx.bumps.get("collaborator").unwrap()],
        ];

        anchor_lang::system_program::create_account(
            anchor_lang::context::CpiContext::new_with_signer(
                ctx.accounts.system_program.to_account_info(),
                anchor_lang::system_program::CreateAccount {
                    from: ctx.accounts.fee_vault_wallet.to_account_info().clone(),
                    to: ctx.accounts.collaborator.to_account_info().clone(),
                },
                &[&fee_vault_wallet_seeds[..], &collaborator_seeds[..]],
            ),
            Rent::get()?.minimum_balance(Collaborator::SIZE),
            Collaborator::SIZE.try_into().unwrap(),
            &ctx.program_id,
        )?;

        if utils::is_discriminator_already_set(&ctx.accounts.collaborator)? {
            return Err(anchor_lang::prelude::ErrorCode::AccountDiscriminatorAlreadySet.into());
        }

        let mut collaborator: Collaborator =
            utils::try_deserialize_unchecked(&ctx.accounts.collaborator)?;
        collaborator.bump = *ctx.bumps.get("collaborator").unwrap();
        collaborator.try_write(&ctx.accounts.collaborator)?;

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(amount: u64)]
pub struct CreateFeeVault<'info> {
    pub system_program: Program<'info, System>,
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        init,
        space = FeeVault::SIZE,
        payer = authority,
        seeds = [
            b"fee_vault".as_ref(),
            authority.key().as_ref(),
        ],
        bump,
    )]
    pub fee_vault: Account<'info, FeeVault>,
    #[account(
        mut,
        seeds = [
            b"fee_vault_wallet".as_ref(),
            fee_vault.key().as_ref(),
        ],
        bump,
    )]
    pub fee_vault_wallet: SystemAccount<'info>,
}

#[derive(Accounts)]
#[instruction(amount: u64)]
pub struct DepositInFeeVault<'info> {
    pub system_program: Program<'info, System>,
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        seeds = [
            b"fee_vault".as_ref(),
            authority.key().as_ref(),
        ],
        bump = fee_vault.bump,
    )]
    pub fee_vault: Account<'info, FeeVault>,
    #[account(
        mut,
        seeds = [
            b"fee_vault_wallet".as_ref(),
            fee_vault.key().as_ref(),
        ],
        bump = fee_vault.wallet_bump,
    )]
    pub fee_vault_wallet: SystemAccount<'info>,
}

#[derive(Accounts)]
#[instruction(amount: u64)]
pub struct WithdrawFromFeeVault<'info> {
    pub system_program: Program<'info, System>,
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        seeds = [
            b"fee_vault".as_ref(),
            authority.key().as_ref(),
        ],
        bump = fee_vault.bump,
    )]
    pub fee_vault: Account<'info, FeeVault>,
    #[account(
        mut,
        seeds = [
            b"fee_vault_wallet".as_ref(),
            fee_vault.key().as_ref(),
        ],
        bump = fee_vault.wallet_bump,
    )]
    pub fee_vault_wallet: SystemAccount<'info>,
}

#[derive(Accounts)]
pub struct CreateCollaborator<'info> {
    pub system_program: Program<'info, System>,
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        seeds = [
            b"fee_vault".as_ref(),
            authority.key().as_ref(),
        ],
        bump = fee_vault.bump,
    )]
    pub fee_vault: Account<'info, FeeVault>,
    #[account(
        mut,
        seeds = [
            b"fee_vault_wallet".as_ref(),
            fee_vault.key().as_ref(),
        ],
        bump = fee_vault.wallet_bump,
    )]
    pub fee_vault_wallet: SystemAccount<'info>,
    /// CHECK: This account is used only as a base for derivation
    pub collaborator_base: UncheckedAccount<'info>,
    /// CHECK: This account is created in this instruction
    #[account(
        mut,
        seeds = [
            b"collaborator".as_ref(),
            fee_vault.key().as_ref(),
            collaborator_base.key().as_ref(),
        ],
        bump
    )]
    pub collaborator: UncheckedAccount<'info>,
}

#[account]
pub struct FeeVault {
    pub bump: u8,
    pub wallet_bump: u8,
}

impl FeeVault {
    pub const SIZE: usize = 8 + 1 + 1;
}

#[account]
pub struct Collaborator {
    pub bump: u8,
}

impl Collaborator {
    pub const SIZE: usize = 8 + 1;

    fn try_write<'info>(&mut self, account: &UncheckedAccount<'info>) -> Result<()> {
        let mut data = account.try_borrow_mut_data()?;
        let dst: &mut [u8] = &mut data;
        let mut cursor = std::io::Cursor::new(dst);
        self.try_serialize(&mut cursor)
    }
}
