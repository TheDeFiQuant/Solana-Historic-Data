use mongodb::{Client, options::ClientOptions};
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;
use serde_json::json;

#[tokio::main] 
async fn main() {
    // Set up MongoDB connection
    let client_options = ClientOptions::parse("mongodb://localhost:27017").await.unwrap();
    let client = Client::with_options(client_options).unwrap();
    let db = client.database("SolanaData");
    let collection = db.collection("AccountTransactions");

    // Set up Solana RPC client and fetch data
    let rpc_client = RpcClient::new("https://api.mainnet-beta.solana.com".to_string());
    let account_pubkey = Pubkey::from_str("PASTE_PUBLIC_KEY_OF_A_SPECIFIC_ACCOUNT_HERE").unwrap();

    match rpc_client.get_signatures_for_address(&account_pubkey) {
        Ok(transaction_history) => {
            let json_history = json!({ "transactions": transaction_history });
    
            // Insert data into MongoDB
            let insert_result = collection.insert_one(json_history, None).await.unwrap();
            println!("New document added with ID: {:?}", insert_result.inserted_id);
        }
        Err(e) => eprintln!("Error fetching transaction history: {:?}", e),
    }
}
