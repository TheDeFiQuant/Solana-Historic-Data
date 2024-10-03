# Tx History Data

This repository provides a comprehensive solution to one of the most common tasks faced by data scientists, data engineers, analysts, and anyone working with Solana blockchain data: gathering and making sense of the vast amount of on-chain data. With billions of transactions recorded on Solana, these transactions form the backbone of data analysis on the blockchain.

The tools provided in this repo help you first understand the number of transactions associated with a specific account. It then enables you to pull detailed, raw transaction data of every single transaction in that account and store it efficiently in a MongoDB database. This approach makes it easier to analyze and derive insights from historic data on any account on Solana.

The repo consists of two main binaries:

- `total_tx`: Fetches and counts the total number of transactions associated with a specified account and stores the signatures of each transaction in MongoDB.
- `main`: Fetches raw transaction data for each signature and stores this data in a separate collection in MongoDB.

## Table of Contents

- [Installation](#installation)
- [Configuration](#configuration)
- [Usage](#usage)
  - [Running `total_tx`](#running-total_tx)
  - [Running `main`](#running-main)
- [Logging](#logging)

## Installation

1. **Clone the repository:**
   ```bash
   git clone https://github.com/TheDeFiQuant/Solana-Historic-Data
   cd tx_history_data
   ```
2. **Install the required dependencies:**  
   Ensure you have Rust and Cargo installed on your system. Then run:
   ```bash
   cargo build --release
   ```
3. **Set up MongoDB:**
   - Ensure you have a MongoDB instance running.
   - Create the database and collections you intend to use.

## Configuration

### Environment Variables

Create a `.env` file in the root directory of the project with the following content:

```env
RPC_URL=YOUR_SOLANA_RPC_URL
MONGO_URL=YOUR_MONGODB_URL
MONGO_DB_NAME=YOUR_DATABASE_NAME
MONGO_TRANSACTION_DATA_COLLECTION=YOUR_TRANSACTION_DATA_COLLECTION_NAME
MONGO_SIGNATURE_COLLECTION=YOUR_SIGNATURE_COLLECTION_NAME
ACCOUNT_PUBKEY=YOUR_SOLANA_ACCOUNT_PUBLIC_KEY
```

Replace each placeholder with your actual values:

- **RPC_URL**: Your Solana RPC endpoint URL.
- **MONGO_URL**: MongoDB connection string.
- **MONGO_DB_NAME**: The name of the MongoDB database.
- **MONGO_TRANSACTION_DATA_COLLECTION**: Collection name for storing transaction signatures.
- **ACCOUNT_PUBKEY**: The public key of the Solana account you wish to track.

## Usage

### Running `total_tx`

To run `total_tx`:

```bash
cargo run --bin total_tx --release
```

## Running `main`

```bash
cargo run --bin main --release
```

### Running as a Systemd Service

To run the application as a service on Linux, create a systemd service file:

1. **Create the service file**: `/etc/systemd/system/tx_history_data.service`

    ```ini
    [Unit]
    Description=Tx History Data Service
    After=network.target

    [Service]
    Type=simple
    User=YOUR_USER
    ExecStart=/usr/local/bin/tx_history_data
    WorkingDirectory=/path/to/your/project
    Environment="PATH=/usr/local/bin"
    Restart=always

    [Install]
    WantedBy=multi-user.target
    ```

2. **Reload the systemd daemon:**

    ```bash
    sudo systemctl daemon-reload
    ```

3. **Enable and start the service:**

    ```bash
    sudo systemctl enable tx_history_data
    sudo systemctl start tx_history_data
    ```

4. **Check the status of the service:**

    ```bash
    sudo systemctl status tx_history_data
    ```

## Logging

The project uses the `flexi_logger` crate to manage logging. Logs are stored in a `logs` directory, and messages of `info`, `warn`, and `error` levels are printed to the console with timestamps.

To view the logs in real-time:

```bash
tail -f /path/to/your/logs/tx_history_data.log
```

Or, if running as a service, check the journal:

```bash
journalctl -u tx_history_data -f
```
