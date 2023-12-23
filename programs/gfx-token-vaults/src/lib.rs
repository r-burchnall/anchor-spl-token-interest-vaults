use anchor_lang::prelude::*;
use anchor_lang::system_program;
use anchor_spl::token::{Mint, Token, TokenAccount};
use anchor_spl::token;

declare_id!("145CK1g8wC9bYZ5fj6qw5KTrxYAAvCTaosrCdhw15S9u");

#[program]
pub mod gfx_token_vaults {
    use super::*;

    #[error_code]
    pub enum Errors {
        #[msg("You have insufficient funds")]
        InsufficientFunds,
        #[msg("This vault has already been initialized")]
        AlreadyInitialized,
    }

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        if ctx.accounts.vault.mint_address != Pubkey::default() {
            return err!(Errors::AlreadyInitialized);
        }

        let interest_vault = &mut ctx.accounts.vault;
        interest_vault.mint_address = ctx.accounts.mint.key();
        interest_vault.owner = *ctx.accounts.owner.key;
        interest_vault.balance = 0;
        interest_vault.ata_address = ctx.accounts.new_ata.key();

        Ok(())
    }

    pub fn withdraw(ctx: Context<Transaction>, val: u64) -> Result<()> {
        if ctx.accounts.vault.balance < val {
           return err!(Errors::InsufficientFunds);
        }
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token::Transfer {
                    from: ctx.accounts.from_ata.to_account_info(),
                    to: ctx.accounts.to_ata.to_account_info(),
                    authority: ctx.accounts.signer.to_account_info()
                },
            ),
            val,
        )?;

        ctx.accounts.from_ata.reload()?;

        ctx.accounts.vault.balance = ctx.accounts.from_ata.amount;

        Ok(())
    }

    pub fn deposit(ctx: Context<Transaction>, val: u64) -> Result<()> {
        msg!("depositing {} tokens", val);
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token::Transfer {
                    from: ctx.accounts.from_ata.to_account_info(),
                    to: ctx.accounts.to_ata.to_account_info(),
                    authority: ctx.accounts.signer.to_account_info()
                },
            ),
            val,
        )?;

        ctx.accounts.to_ata.reload()?;

        ctx.accounts.vault.balance = ctx.accounts.to_ata.amount;

        Ok(())
    }

    pub fn apply_interest(ctx: Context<Transaction>) -> Result<()> {
        let interest = ((ctx.accounts.to_ata.amount * 101) / 100) - ctx.accounts.to_ata.amount;
        msg!("applying interest of {} tokens", interest);
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token::Transfer {
                    from: ctx.accounts.from_ata.to_account_info(),
                    to: ctx.accounts.to_ata.to_account_info(),
                    authority: ctx.accounts.signer.to_account_info()
                },
            ),
            interest,
        )?;

        ctx.accounts.to_ata.reload()?;
        ctx.accounts.vault.balance = ctx.accounts.to_ata.amount;

        Ok(())
    }
}

#[account]
#[derive(InitSpace)]
pub struct InterestVault {
    balance: u64,
    mint_address: Pubkey,
    ata_address: Pubkey,
    owner: Pubkey, // owner of the vault - to prevent withdrawals from other users
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
      init,
      seeds = [b"vault", mint.key().as_ref(), owner.key().as_ref()],
      bump,
      payer = owner,
      space = 8 + InterestVault::INIT_SPACE
    )]
    pub vault: Account<'info, InterestVault>,

    #[account(
    init,
    payer = owner,
    token::mint = mint,
    token::authority = owner,
    seeds = [mint.key().as_ref(), vault.key().as_ref()],
    bump,
    )]
    pub new_ata: Account<'info, TokenAccount>,
    pub mint: Account<'info, Mint>,

    #[account(mut)]
    pub owner: Signer<'info>,
    pub rent: Sysvar<'info, Rent>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>
}


#[derive(Accounts)]
pub struct Transaction<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(mut)]
    pub vault: Account<'info, InterestVault>,
    #[account(mut)]
    pub to_ata: Account<'info,TokenAccount>,
    #[account(mut)]
    pub from_ata: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

