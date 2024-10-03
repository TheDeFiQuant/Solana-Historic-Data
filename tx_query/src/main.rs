use flexi_logger::{Logger, WriteMode, FileSpec, Duplicate, LogSpecification, Criterion, Naming, Cleanup};
use log::{info, warn, error, Record};
use mongodb::{Client, options::ClientOptions, bson::{doc, Bson, Document}};
use solana_client::rpc_client::{RpcClient, GetConfirmedSignaturesForAddress2Config};
use solana_sdk::{pubkey::Pubkey, signature::Signature};
use solana_client::rpc_config::RpcTransactionConfig;
use solana_transaction_status::UiTransactionEncoding;
use std::str::FromStr;
use futures::stream::StreamExt;
use std::collections::HashSet;
use std::env;
use dotenv::dotenv;
use std::time::Duration;
use tokio::time::sleep;

const INITIAL_RETRY_DELAY: u64 = 2; // Initial delay in seconds for retry
const MAX_RETRY_DELAY: u64 = 600; // Maximum delay in seconds (10 minutes)

#[tokio::main]
async fn main() {
    // Load environment variables
    dotenv().ok();

    // Initialize the logger with timestamps and other relevant information
    Logger::try_with_str("info") // Set logging level to `Info` (includes `warn` and `error`)
        .unwrap()
        .log_to_file(FileSpec::default().directory("logs").suffix("log")) // Log directory for error logs
        .write_mode(WriteMode::BufferAndFlush)
        .append() // Append to the existing log file
        .use_utc() // Use UTC to avoid IndeterminateOffset error
        .duplicate_to_stderr(Duplicate::Info) // Print `info`, `warn`, and `error` log levels to the console
        .format(flexi_logger::detailed_format) // Use detailed format to include timestamps
        .start()
        .unwrap();

    // Test logger initialization
    info!("Logger successfully started!");

    // Get environment variables for configuration
    let rpc_url = env::var("RPC_URL").expect("RPC_URL must be set in the environment variables.");
    let mongo_url = env::var("MONGO_URL").expect("MONGO_URL must be set in the environment variables.");
    let mongo_db_name = env::var("MONGO_DB_NAME").expect("MONGO_DB_NAME must be set in the environment variables.");
    let transaction_data_collection_name = env::var("MONGO_TRANSACTION_DATA_COLLECTION").expect("MONGO_TRANSACTION_DATA_COLLECTION must be set in the environment variables.");
    let signature_collection_name = env::var("MONGO_SIGNATURE_COLLECTION").expect("MONGO_SIGNATURE_COLLECTION must be set in the environment variables.");
    let account_pubkey_str = env::var("ACCOUNT_PUBKEY").expect("ACCOUNT_PUBKEY must be set in the environment variables.");

    // Set up MongoDB connection
    info!("Connecting to MongoDB at {}...", mongo_url);
    let client_options = ClientOptions::parse(&mongo_url).await.expect("Failed to parse MongoDB client options.");
    let mongo_client = Client::with_options(client_options).expect("Failed to create MongoDB client.");
    let db = mongo_client.database(&mongo_db_name);
    let transaction_data_collection = db.collection::<Document>(&transaction_data_collection_name);
    let signature_collection = db.collection::<Document>(&signature_collection_name);
    info!(
        "Connected to MongoDB and selected the '{}' database and '{}' and '{}' collections.",
        mongo_db_name, transaction_data_collection_name, signature_collection_name
    );

    // Set up Solana RPC client
    info!("Connecting to Solana RPC at {}", rpc_url);
    let rpc_client = RpcClient::new(rpc_url);
    let account_pubkey = Pubkey::from_str(&account_pubkey_str).expect("Invalid public key");
    info!("Using account pubkey: {}", account_pubkey);

    // Initialize variables for pagination and processing
    let mut total_transactions = 0;
    let mut before_signature: Option<Signature> = None;

    // Fetch all existing signatures from MongoDB
    let mut existing_signatures = HashSet::new();
    let mut cursor = signature_collection.find(doc! {}, None).await.expect("Failed to fetch existing signatures from MongoDB");

    while let Some(result) = cursor.next().await {
        match result {
            Ok(document) => {
                if let Ok(signature) = document.get_str("signature") {
                    existing_signatures.insert(signature.to_string());
                }
            }
            Err(e) => error!("Failed to read document: {:?}", e),
        }
    }

    info!("Fetched {} existing signatures from MongoDB.", existing_signatures.len());

    // Loop to fetch all transaction signatures for the given account
    info!("Fetching transaction signatures...");
    loop {
        // Log current pagination state before fetching
        info!("About to fetch transaction signatures; current before_signature: {:?}", before_signature);

        // Fetch a batch of transaction signatures
        let signatures = rpc_client
            .get_signatures_for_address_with_config(
                &account_pubkey,
                GetConfirmedSignaturesForAddress2Config {
                    before: before_signature.clone(),
                    limit: Some(1000),
                    ..Default::default()
                },
            )
            .expect("Failed to fetch signatures");

        if signatures.is_empty() {
            info!("No more signatures found. Exiting loop.");
            break;
        }

        // Update the before_signature to paginate through all transactions
        before_signature = Some(Signature::from_str(&signatures.last().unwrap().signature).unwrap());
        info!("Updated before_signature for pagination: {:?}", before_signature);

        // Filter out signatures that are already in MongoDB
        let new_signatures: Vec<_> = signatures
            .into_iter()
            .filter(|s| !existing_signatures.contains(&s.signature))
            .collect();

        // If new_signatures is empty, continue to the next batch
        if new_signatures.is_empty() {
            info!("No new signatures found in this batch.");
            continue;
        }

        // Update the total transaction count
        total_transactions += new_signatures.len();
        info!("Fetched {} new transactions so far...", total_transactions);

        // Process each new transaction signature
        for signature_info in &new_signatures {
            let sig = Signature::from_str(&signature_info.signature).unwrap();

            info!("Fetching transaction data for signature: {}", signature_info.signature);

            // Retry logic for fetching transaction
            let mut retry_count = 0;
            let mut success = false;

            while !success {
                // Create config for fetching transactions with supported version
                let config = RpcTransactionConfig {
                    encoding: Some(UiTransactionEncoding::Json),
                    max_supported_transaction_version: Some(0),
                    ..Default::default()
                };

                // Fetch detailed transaction data with maxSupportedTransactionVersion
                match rpc_client.get_transaction_with_config(&sig, config) {
                    Ok(transaction) => {
                        info!("Successfully fetched transaction data for {}", signature_info.signature);
                        let document = mongodb::bson::to_document(&transaction).unwrap();

                        // Insert into TransactionData collection
                        let insert_result = transaction_data_collection.insert_one(document, None).await.expect("Failed to insert document");
                        info!("Inserted transaction with signature: {} and MongoDB ID: {:?}", signature_info.signature, insert_result.inserted_id);

                        // Insert signature into Signatures collection
                        signature_collection.insert_one(doc! {
                            "signature": signature_info.signature.clone(),
                            "slot": Bson::Int64(signature_info.slot as i64),
                            "err": signature_info.err.clone().map(|e| Bson::String(e.to_string())),
                            "memo": signature_info.memo.clone(),
                            "block_time": signature_info.block_time.map(|bt| Bson::Int64(bt as i64)),
                            "confirmation_status": signature_info.confirmation_status.clone().map(|cs| Bson::String(format!("{:?}", cs)))
                        }, None).await.expect("Failed to insert signature document");

                        // Mark as successful to break out of retry loop
                        success = true;
                    },
                    Err(e) => {
                        retry_count += 1;
                        let delay = (INITIAL_RETRY_DELAY * 2u64.pow(retry_count - 1)).min(MAX_RETRY_DELAY);
                        error!("Error fetching transaction for signature: {}: {:?} (retrying in {} seconds)", signature_info.signature, e, delay);
                        sleep(Duration::from_secs(delay)).await;
                    },
                }
            }
        }
    }

    // Final total transaction count
    info!("Total transactions processed: {}", total_transactions);
}
