use anchor_lang::Accounts;
use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Mint};
use crate::interestvault::InterestVault;

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

