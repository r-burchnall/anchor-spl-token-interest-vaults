use solana_client::{
    rpc_client::RpcClient,
};
use solana_client::nonce_utils::get_account;
use solana_sdk::{commitment_config::CommitmentConfig, pubkey};
use solana_sdk::instruction::{Instruction,AccountMeta};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use solana_sdk::transaction::Transaction;
use solana_sdk::{
    system_program,
};
const PROGRAM_ID: Pubkey = pubkey!("145CK1g8wC9bYZ5fj6qw5KTrxYAAvCTaosrCdhw15S9u");
const TOKEN_PROGRAM_ID: Pubkey = pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");

fn main() -> Result<(), ()> {
    let interest_payer_keypair = Keypair::new();

    let rpc_url = "http://localhost:8899".to_string();
    let connection = RpcClient::new_with_commitment(rpc_url.to_string(), CommitmentConfig::confirmed());

    let accounts = connection.get_program_accounts(&PROGRAM_ID);
    if Err(accounts) {
        println!("Error: {:?}", accounts);
        return Err(());
    }

    let non_blocking_client = solana_client::nonblocking::rpc_client::RpcClient::new_with_commitment(
        rpc_url.to_string(),
        CommitmentConfig::confirmed(),
    );

    let accounts = accounts.unwrap();
    for account in accounts {
        let vault_pubkey = account.0;
        let account = account.1;

        // await getAssociatedTokenAddress(mint.address, userWallet.publicKey),
        let ata = non_blocking_client.get_account();
        let instruction = Instruction::new_with_borsh(
            PROGRAM_ID,
            Default::default(),
            vec![
                AccountMeta::new(interest_payer_keypair.pubkey(), true), // signer
                AccountMeta::new(vault_pubkey, false), // vault
                AccountMeta::new(account.data, false), //toata
                AccountMeta::new(account.data, false), //fromata
                AccountMeta::new(TOKEN_PROGRAM_ID, false) //token_program
            ],
        );


    }

    Ok(())
}

fn send_initialize_tx(
    client: &RpcClient,
    program_id: Pubkey,
    payer: &Keypair
) -> Result<()> {

    let ix = gfx_token_vaults::transaction::Transaction;

    let instruction = Instruction::new_with_borsh(
        program_id,
        &ix,
        vec![

        ],
    );

    let blockhash = client.get_latest_blockhash()?;
    let mut tx = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&payer.pubkey()),
        &[payer],
        blockhash,
    );
    client.send_and_confirm_transaction(&tx)?;

    Ok(())
}