
# MCS Binary Project

## Overview

**MCS Binary** is a Rust-based project implementing a client-server architecture for processing, signing, and aggregating data over WebSocket connections. Its primary use case is handling Bitcoin (BTC/USDT) trade prices, securely signing messages using Ed25519 cryptographic signatures, and computing aggregated statistics like averages on a centralized server.

This project consists of two main components:
- **Client**: Fetches trade prices, computes averages, signs messages, and communicates with the aggregator.
- **Aggregator**: A WebSocket server that receives signed data from clients, verifies signatures, computes global averages, and manages client connections.

---

## Features

- Cryptographic signing and verification using **Ed25519** (`ed25519-dalek`).
- Real-time WebSocket communication enabled by **tokio-tungstenite**.
- JSON-based configuration and data handling using **serde** and **serde_json**.
- Modular and scalable design for concurrent client-server communication.
- Command-line argument parsing with **clap** for dynamic configuration.

---

## Project Structure

```plaintext
mcs_binary/
├── Cargo.toml          # Project metadata and dependencies
├── src/
│   ├── client/
│   │   ├── main.rs     # Entry point for client binary
│   │   ├── client.rs   # Client logic (WebSocket communication, signing, and sending data)
│   ├── aggregator/
│   │   ├── main.rs     # Entry point for aggregator binary
│   │   ├── aggregator.rs # Aggregator logic (verification and aggregation)
│   ├── utils.rs        # Shared utility functions (key management, CLI parsing, etc.)
```

---

## Dependencies

- **tokio**: Asynchronous runtime for Rust.
- **tokio-tungstenite**: WebSocket communication framework.
- **serde / serde_json**: For JSON serialization and deserialization.
- **clap**: Command-line argument parser.
- **ed25519-dalek**: Cryptographic library for Ed25519 signatures.
- **base64**: Encoding and decoding for data serialization.
- **futures**: Async utilities for concurrent programming.

---

## Usage

### Build and Run

1. Clone the repository:
   ```bash
   git clone https://github.com/wdcs-pruthvithakor/rust-mcs-multi-binary.git
   cd rust-mcs-multi-binary
   ```

2. Build the project:
   ```bash
   cargo build --release
   ```

3. Run the Aggregator:
   ```bash
   ./target/release/aggregator
   ```

4. In new terminal start client:

   ```bash
   ./target/release/client --mode=cache --times=10
   ```

---

### Command-Line Options

#### **Client Binary**
- **`--mode`**: Operation mode for the client. Options:
  - `cache`: Fetches BTC/USDT prices, computes averages, and sends data to the aggregator.
  - `read`: Reads and displays previously saved data.
- **`--times`**: Duration in seconds for fetching prices from the WebSocket. Default: `1`.

#### **Aggregator Binary**
No specific arguments; the server listens on `ws://127.0.0.1:8080` for incoming connections.

---

## Functionality Breakdown

### Utilities (`utils.rs`)
- **Key Management**:
  - Loads Ed25519 private keys from `client_keys.json`.
- **Data Handling**:
  - Processes WebSocket messages to extract price data.
  - Computes averages and saves results to files.
- **CLI Argument Parsing**:
  - Handles runtime configuration such as mode and duration.

### Client (`client.rs`)
- Connects to a WebSocket server (e.g., `ws://127.0.0.1:8080`).
- Fetches BTC/USDT trade prices, calculates averages, and signs data.
- Sends signed messages to the aggregator.

### Aggregator (`aggregator.rs`)
- Runs a WebSocket server on `ws://127.0.0.1:8080`.
- Receives and verifies messages from clients.
- Computes a global average of BTC prices and saves aggregated results.

---

## Configuration


### Key Management

The project uses Ed25519 private and public keypairs for secure signing and verification. These keypairs are stored in a JSON file (`client_keys.json`) with the following structure:

#### Key File Format: `client_keys.json`

```json
[
    {
        "private_key": "<base64-encoded-private-key>",
        "public_key": "<base64-encoded-public-key>"
    },
    {
        "private_key": "<base64-encoded-private-key>",
        "public_key": "<base64-encoded-public-key>"
    },
    ...
]
```

### Key File Usage

- **Clients**: Use their respective private keys to sign messages before sending them to the aggregator.
- **Aggregator**: Verifies each incoming message's signature using the corresponding public keys.


### WebSocket Configuration
- **Aggregator**: Listens on `ws://127.0.0.1:8080`.
- **Client**: Connects to the local aggregator or a public WebSocket endpoint for BTC price data.

### Adjustable Parameters
- **Number of Clients**: Modify in the code or run multiple client instances.
- **Data Files**:
  - Clients save their data as `client_<id>_data.txt`.
  - The aggregator saves results in `global_data.txt`.

---

## Sample Outputs

### Aggregator
```bash
cargo run --bin aggregator
```
Output:
```plaintext
Aggregator WebSocket server listening on ws://127.0.0.1:8080
New client connected!
Verified message from Client-1: 96650.2884
Verified message from Client-3: 96654.7298
Aggregator: Global average BTC price: 96651.7868
```

### Client
```bash
cargo run --bin client -- --mode=cache --times=5
```
Output:
```plaintext
Will listen for 5 seconds.
Client 1: Connected to WebSocket.
Client 1: Average BTC price: 96650.2884
...
Received from server: Global average BTC price: 96651.7868
```

---

## Examples

### Start the Aggregator
```bash
cargo run --bin aggregator
```

### Start Multiple Clients
```bash
cargo run --bin client -- --mode=cache --times=6
```

### Read Cached Data
```bash
cargo run --bin client -- --mode=read
```

<!-- ---

## Future Enhancements

- Dynamic client registration for increased scalability.
- Enhanced error handling for network interruptions.
- Support for additional cryptocurrency price streams.
- Integration with a database for historical data storage and analysis.

--- -->

## License

This project is licensed under the MIT License. See the `LICENSE` file for details.
