use serde_json::{Value, json};
use mongodb::{Client, bson};
use solana_client::rpc_client::RpcClient;
use solana_client::rpc_config::RpcTransactionConfig;
use solana_sdk::signature::Signature;
use std::str::FromStr;

#[tokio::main]
async fn main() -> mongodb::error::Result<()> {
    let client = RpcClient::new("https://api.mainnet-beta.solana.com".to_string());
    let signature = Signature::from_str("PASTE_PUBLIC_KEY_OF_A_SPECIFIC_TRANSACTION_HERE").expect("Invalid signature");

    let config = RpcTransactionConfig {
        max_supported_transaction_version: Some(0),
        ..Default::default()
    };

    let transaction = client.get_transaction_with_config(&signature, config)
        .expect("Failed to fetch transaction");
    
    // Convert the transaction data to JSON Value
    let transaction_json: Value = serde_json::from_str(&serde_json::to_string(&transaction).unwrap()).unwrap();

    // Extract specific fields
    let extracted_data = json!({
        "blockTime": transaction_json["blockTime"],
        "computeUnitsConsumed": transaction_json["meta"]["computeUnitsConsumed"],
        "err": transaction_json["meta"]["err"],
        "fee": transaction_json["meta"]["fee"],
        "Err": transaction_json["meta"]["status"]["Err"],
        "logMessages": transaction_json["meta"]["logMessages"],
        "slot": transaction_json["slot"],
        "signatures": transaction_json["transaction"]["signatures"]
    });

    // Connect to MongoDB
    let client = Client::with_uri_str("mongodb://localhost:27017").await?;
    let database = client.database("SolanaData");
    let collection = database.collection("TransactionDetails");

    // Convert the extracted data to a document
    let transaction_doc = bson::to_document(&extracted_data).unwrap();

    // Insert the document into the collection
    let insert_result = collection.insert_one(transaction_doc, None).await?;

    println!("New document added with ID: {:?}", insert_result.inserted_id);
    Ok(())
}
