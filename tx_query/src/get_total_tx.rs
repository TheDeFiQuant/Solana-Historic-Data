use mongodb::{Client, options::ClientOptions, bson::{doc, Document}};
use solana_client::rpc_client::{RpcClient, GetConfirmedSignaturesForAddress2Config};
use solana_sdk::{pubkey::Pubkey, signature::Signature};
use std::str::FromStr;
use std::collections::HashSet;
use std::env;
use dotenv::dotenv;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() {
    // Load environment variables
    dotenv().ok();

    // Get environment variables for configuration
    let rpc_url = env::var("RPC_URL").expect("RPC_URL must be set in the environment variables.");
    let mongo_url = env::var("MONGO_URL").expect("MONGO_URL must be set in the environment variables.");
    let mongo_db_name = env::var("MONGO_DB_NAME").expect("MONGO_DB_NAME must be set in the environment variables.");
    let signature_collection_name = env::var("MONGO_SIGNATURE_COLLECTION").expect("MONGO_SIGNATURE_COLLECTION must be set in the environment variables.");
    let account_pubkey_str = env::var("ACCOUNT_PUBKEY").expect("ACCOUNT_PUBKEY must be set in the environment variables.");

    // Set up MongoDB connection
    println!("Connecting to MongoDB at {}...", mongo_url);
    let client_options = ClientOptions::parse(&mongo_url).await.expect("Failed to parse MongoDB client options.");
    let mongo_client = Client::with_options(client_options).expect("Failed to create MongoDB client.");
    let db = mongo_client.database(&mongo_db_name);
    let signature_collection = db.collection::<Document>(&signature_collection_name);
    println!(
        "Connected to MongoDB and selected the '{}' database and '{}' collection.",
        mongo_db_name, signature_collection_name
    );

    // Count the total signatures already stored in the MongoDB collection
    let mut total_signatures_in_db = signature_collection.count_documents(doc! {}, None)
        .await
        .expect("Failed to count documents in MongoDB.");
    println!("Total signatures already in {}: {}", signature_collection_name, total_signatures_in_db);

    // Set up Solana RPC client
    let client = RpcClient::new(rpc_url.to_string());
    let account_pubkey = Pubkey::from_str(&account_pubkey_str).expect("Invalid public key");

    let mut before_signature = None;

    // Create a set to track unique signatures already in memory
    let mut existing_signatures = HashSet::new();

    // Loop to fetch transaction signatures for the account
    loop {
        let mut retry_delay = Duration::from_secs(5); // Start with 5 seconds delay
        let max_delay = Duration::from_secs(600); // Max delay of 10 minutes

        let signatures = loop {
            match client.get_signatures_for_address_with_config(
                &account_pubkey,
                GetConfirmedSignaturesForAddress2Config {
                    before: before_signature.clone(),
                    limit: Some(1000),  // Adjust the limit per your API limits or needs
                    ..Default::default()
                },
            ) {
                Ok(signatures) => break signatures, // Break the loop on successful fetch
                Err(err) => {
                    // Log error and retry
                    eprintln!("Error fetching signatures: {:?}. Retrying in {:?}...", err, retry_delay);

                    // Wait for the retry delay before attempting again
                    sleep(retry_delay).await;

                    // Exponentially backoff retry delay (double the delay)
                    retry_delay = std::cmp::min(retry_delay * 2, max_delay);
                }
            }
        };

        if signatures.is_empty() {
            break;
        }

        for signature_info in &signatures {
            let sig = signature_info.signature.clone();

            // Check if signature is already in the set to avoid duplicates
            if !existing_signatures.contains(&sig) {
                // Insert signature into MongoDB
                signature_collection.insert_one(doc! {
                    "signature": &sig,
                    "slot": signature_info.slot as i64,
                    "err": signature_info.err.clone().map(|e| e.to_string()),
                    "memo": signature_info.memo.clone(),
                    "block_time": signature_info.block_time.map(|bt| bt as i64),
                    "confirmation_status": signature_info.confirmation_status.clone().map(|cs| format!("{:?}", cs))
                }, None).await.expect("Failed to insert signature document");

                // Add the signature to the set of existing signatures
                existing_signatures.insert(sig);

                // Increment the count of total signatures in the database
                total_signatures_in_db += 1;
            }
        }

        // Display the updated count of total signatures in the database
        println!("Total signatures in {}: {}", signature_collection_name, total_signatures_in_db);

        // Update the pagination point to fetch the next batch of signatures
        before_signature = Some(Signature::from_str(&signatures.last().unwrap().signature).unwrap());
    }

    println!("Final count of total signatures in {}: {}", signature_collection_name, total_signatures_in_db);
}
