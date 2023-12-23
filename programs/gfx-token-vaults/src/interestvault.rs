use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct InterestVault {
    pub balance: u64,
    pub mint_address: Pubkey,
    pub ata_address: Pubkey,
    pub owner: Pubkey, // owner of the vault - to prevent withdrawals from other users
}