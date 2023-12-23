use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};
use crate::interestvault::InterestVault;

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
