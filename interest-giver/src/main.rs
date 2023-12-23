use solana_client::{
    rpc_client::RpcClient,
    rpc_filter::{RpcFilterType, Memcmp, MemcmpEncodedBytes, MemcmpEncoding},
    rpc_config::{RpcProgramAccountsConfig, RpcAccountInfoConfig},
};
use solana_sdk::{commitment_config::CommitmentConfig, program_pack::Pack, pubkey};
use spl_token::{state::{Mint, Account}};
use solana_account_decoder::{UiAccountEncoding};
use solana_sdk::pubkey::Pubkey;

fn main() -> Result<(), ()> {
    const MY_WALLET_ADDRESS:Pubkey = pubkey!("FriELggez2Dy3phZeHHAdpcoEXkKQVkv6tx3zDtCVP8T");
    const PROGRAM_ID: Pubkey = pubkey!("145CK1g8wC9bYZ5fj6qw5KTrxYAAvCTaosrCdhw15S9u");


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
        let pubkey = account.0;
        let account = account.1;
        let account_data = non_blocking_client.get_account_with_commitment(&pubkey, CommitmentConfig::confirmed()).await.expect("blow");
        if Err(account_data) {
            println!("Error: {:?}", account_data);
            return Err(());
        }
        let account_data = account_data.unwrap();
        let account_data = account_data.value;
        let account_data = account_data.data;
        let account_data = account_data.as_ref();
        let account_data = Account::unpack(account_data);
        if Err(account_data) {
            println!("Error: {:?}", account_data);
            return Err(());
        }
        let account_data = account_data.unwrap();
        let account_data = account_data.amount;
        println!("Account: {:?} has {:?} tokens", account.pubkey(), account_data);
    }

    Ok(())
}

fn send_initialize_tx(
    client: &RpcClient,
    program_id: Pubkey,
    payer: &Keypair
) -> Result<()> {

    let bank_instruction = gfx::Initialize;

    let instruction = Instruction::new_with_borsh(
        program_id,
        &bank_instruction,
        vec![],
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