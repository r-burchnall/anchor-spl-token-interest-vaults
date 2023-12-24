use std::sync::Arc;
use anchor_client::anchor_lang::AccountDeserialize;
use solana_client::{
    rpc_client::RpcClient,
};
use solana_sdk::{commitment_config::CommitmentConfig, pubkey};
use solana_sdk::instruction::{Instruction,AccountMeta};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{read_keypair_file, Signer};
use solana_sdk::transaction::Transaction;
use spl_associated_token_account::get_associated_token_address;

use gfx_token_vaults::interestvault::InterestVault;

const TOKEN_PROGRAM_ID: Pubkey = pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        println!("Usage: interest-giver <interest payer keypair path> <program id>");
        return;
    }

    let interest_payer_keypair_path = args[1].clone();
    let interest_payer_keypair = read_keypair_file(interest_payer_keypair_path.as_str()).unwrap_or_else(|err| {
        println!("Failure to read keypair file: {:?}", err);
        std::process::exit(1);
    });

    let program_id = args[2].parse::<Pubkey>().unwrap_or_else(|_| {
        println!("Program id is not a valid pubkey");
        std::process::exit(1);
    });

    // TODO: allow us to target a specific RPC node
    let rpc_url = "http://localhost:8899".to_string();
    let connection = RpcClient::new_with_commitment(rpc_url.to_string(), CommitmentConfig::confirmed());

    let accounts = connection.get_program_accounts(&program_id).unwrap();

    // create our non-blocking client
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

            let mint_address = account.mint_address;
            let to_ata = account.ata_address;
            let from_ata = get_associated_token_address(&interest_payer_keypair.pubkey(), &mint_address);

            // While this works, it's not the best way to do this
            // Looks like this hex string is the discriminator for the instruction // TODO: confirm this
            let data_as_hex_str = "b9ed3261ecb304bc";
            let data = hex::decode(data_as_hex_str).unwrap();

            let ix = Instruction::new_with_bytes(
                program_id,
                data.as_slice(),
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
                    Ok(x) => println!("Success: {:?}", x),
                    Err(e) => println!("Error for {}: {:?}", vault_pubkey, e),
                }
            })
        })
        .collect::<Vec<_>>();

    futures::future::join_all(requests).await;

    println!("Completed");
}