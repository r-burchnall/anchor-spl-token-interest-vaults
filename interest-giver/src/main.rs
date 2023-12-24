use std::sync::Arc;
use anchor_client::anchor_lang::AccountDeserialize;
use solana_client::{
    rpc_client::RpcClient,
};
use solana_sdk::{commitment_config::CommitmentConfig, pubkey};
use solana_sdk::instruction::{Instruction,AccountMeta};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, Signer};
use solana_sdk::transaction::Transaction;

use gfx_token_vaults::interestvault::InterestVault;
use futures::future::{join_all, ok};
use solana_sdk::account::ReadableAccount;

const PROGRAM_ID: Pubkey = pubkey!("145CK1g8wC9bYZ5fj6qw5KTrxYAAvCTaosrCdhw15S9u");
const TOKEN_PROGRAM_ID: Pubkey = pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");

#[tokio::main]
async fn main() {
    let interest_payer_keypair = Keypair::new();

    let rpc_url = "http://localhost:8899".to_string();
    let connection = RpcClient::new_with_commitment(rpc_url.to_string(), CommitmentConfig::confirmed());

    let accounts = connection.get_program_accounts(&PROGRAM_ID).unwrap();

    let non_blocking_client = solana_client::nonblocking::rpc_client::RpcClient::new_with_commitment(
        rpc_url.to_string(),
        CommitmentConfig::confirmed(),
    );

    let non_blocking_client = Arc::new(non_blocking_client);

    let recent_blockhash = connection.get_latest_blockhash().unwrap();

    let requests = accounts
        .iter()
        .map(|acc| {
            let account = acc.clone();
            let vault_pubkey = account.0;

            let account = account.1;
            let mut buf: &[u8] = account.data.as_slice();
            let account = InterestVault::try_deserialize(&mut buf).unwrap();

            let to_ata = account.ata_address;
            let from_ata = interest_payer_keypair.pubkey();

            let ix = Instruction::new_with_bytes(
                PROGRAM_ID,
                &[0],
                vec![
                    AccountMeta::new(interest_payer_keypair.pubkey(), true), // signer
                    AccountMeta::new(vault_pubkey, false), // vault
                    AccountMeta::new(to_ata, false), //toata
                    AccountMeta::new(from_ata, false), //fromata
                    AccountMeta::new(TOKEN_PROGRAM_ID, false) //token_program
                ],
            );

            let tx: Transaction = Transaction::new_signed_with_payer(
                &[ix],
                Some(&interest_payer_keypair.pubkey()),
                &[&interest_payer_keypair],
                recent_blockhash,
            );

            let client = Arc::clone(&non_blocking_client);
            tokio::task::spawn(async move {
                println!("Applying interest to account: {}", vault_pubkey);
                match client.send_and_confirm_transaction(&tx).await {
                    Ok(_) => println!("Success"),
                    Err(e) => println!("Error for {}: {:?}", vault_pubkey, e),
                }
            })
        })
        .collect::<Vec<_>>();

    futures::future::join_all(requests).await;

    println!("Completed");
}