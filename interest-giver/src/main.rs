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

    let accounts = connection.get_program_accounts(&PROGRAM_ID);
    println!("Accounts: {:?}", accounts);

    let non_blocking_client = solana_client::nonblocking::rpc_client::RpcClient::new_with_commitment(
        rpc_url.to_string(),
        CommitmentConfig::confirmed(),
    );

    // let mut requests = vec![];
    let accounts = accounts.unwrap();
    for account in accounts {
        let vault_pubkey = account.0;
        println!("Applying interest to account: {}", vault_pubkey);

        let account = account.1;
        let mut buf: &[u8] = account.data.as_slice();
        let account = InterestVault::try_deserialize(&mut buf).unwrap();

        let to_ata = account.ata_address;
        let from_ata = account.ata_address;

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

        let recent_blockhash = connection.get_recent_blockhash().unwrap().0;

        let tx: Transaction = Transaction::new_signed_with_payer(
            &[ix],
            Some(&interest_payer_keypair.pubkey()),
            &[&interest_payer_keypair],
            recent_blockhash,
        );

        let tx = non_blocking_client.send_and_confirm_transaction(&tx).await;
        println!("Tx: {:?}", tx);

        // // // let client: Arc<solana_client::nonblocking::rpc_client::RpcClient> = Arc::clone(&non_blocking_client);
        // // let request = tokio::spawn(async move {
        // //     let client = client;
        // //     client.send_and_confirm_transaction(&tx).await;
        // // });
        //
        // requests.push(request);
    }

    println!("Completed");
    // join_all(requests).await;
}
//
// fn send_initialize_tx(
//     client: &RpcClient,
//     program_id: Pubkey,
//     payer: &Keypair
// ) -> Result<()> {
//
//     let ix = gfx_token_vaults::transaction::Transaction;
//
//     let instruction = Instruction::new_with_borsh(
//         program_id,
//         &ix,
//         vec![
//
//         ],
//     );
//
//     let blockhash = client.get_latest_blockhash()?;
//     let mut tx = Transaction::new_signed_with_payer(
//         &[instruction],
//         Some(&payer.pubkey()),
//         &[payer],
//         blockhash,
//     );
//     client.send_and_confirm_transaction(&tx)?;
//
//     Ok(())
// }