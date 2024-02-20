use serde_json::{Value, json};
use mongodb::{Client, bson::doc};
use mongodb::bson;
use solana_client::rpc_client::RpcClient;
use solana_client::rpc_config::RpcTransactionConfig;
use solana_sdk::signature::Signature;
use std::str::FromStr;

#[tokio::main]
async fn main() -> mongodb::error::Result<()> {
    let mongo_client = Client::with_uri_str("mongodb://localhost:27017").await?;
    let database = mongo_client.database("SolanaData");
    let collection = database.collection("TransactionDetails");

    // Fetch the document containing the signatures and handle the Option<Document>
    let result: Option<mongodb::bson::Document> = database.collection("MergedAccountTransactions")
        .find_one(Some(doc! {}), None)
        .await?;

    if let Some(signatures_doc) = result {
        let signatures = signatures_doc.get_array("transactions")
            .expect("Failed to get transactions array");

        let solana_client = RpcClient::new("https://api.mainnet-beta.solana.com".to_string());

    for signature_value in signatures {
        let signature_str = signature_value.as_document().unwrap().get_str("signature").unwrap();
        let signature = Signature::from_str(signature_str).expect("Invalid signature");
    
        let config = RpcTransactionConfig {
            max_supported_transaction_version: Some(0),
            ..Default::default()
        };
    
        let transaction = solana_client.get_transaction_with_config(&signature, config)
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

        // Convert the extracted data to a document
        let transaction_doc = bson::to_document(&extracted_data).unwrap();

        // Insert the document into the collection
        let insert_result = collection.insert_one(transaction_doc, None).await?;


        println!("Processed signature: {}", signature_str);
    }

    } else {
        println!("No document found with signatures.");
    }

    Ok(())
}
