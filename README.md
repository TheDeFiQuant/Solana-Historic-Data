# Solana Transaction Query Tool

This tool provides functionality for querying Solana transactions and storing relevant details in a MongoDB database. It consists of several components, each responsible for a different aspect of transaction processing.

## Prerequisites

Before setting up the project, ensure you have MongoDB installed and running on your system. The application connects to MongoDB on the default port (`27017`).

## Setup

1. **Install MongoDB**: Follow the official MongoDB documentation to install MongoDB on your system if you haven't already.

2. **Configure MongoDB**: The application expects a MongoDB instance running on `mongodb://localhost:27017`. Ensure MongoDB is configured to accept connections on this URI.

3. **Install Rust**: The project is developed in Rust. If Rust is not installed on your system, follow the [official Rust documentation](https://www.rust-lang.org/tools/install) to set it up.

## Running the Applications

The source code contains three main applications:

- `main.rs`: Processes transactions and stores their details in the `TransactionDetails` collection.
- `tx_details.rs`: Fetches details of a specific transaction based on a provided public key and stores it in the `TransactionDetails` collection.
- `tx_history.rs`: Fetches the transaction history for a specific account based on the public key and stores it in the `AccountTransactions` collection.

To run any of these applications, navigate to the `src` folder and execute the following command:

cargo run --bin <application-name>


Replace `<application-name>` with `main`, `tx_details`, or `tx_history` depending on which application you wish to run.

## Adding Public Keys

- For `tx_details.rs`, replace `"PASTE_PUBLIC_KEY_OF_A_SPECIFIC_TRANSACTION_HERE"` with the public key of the transaction you are interested in.
- For `tx_history.rs`, replace `"PASTE_PUBLIC_KEY_OF_A_SPECIFIC_ACCOUNT_HERE"` with the public key of the account whose transaction history you want to fetch.

These keys need to be manually added to the respective files before running the applications.

## Dependencies

The project dependencies are listed in `Cargo.toml`. They include `solana-client`, `solana-sdk`, `serde_json`, `mongodb`, and `tokio`.
