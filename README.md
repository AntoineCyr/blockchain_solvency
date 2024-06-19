# Zero-Knowledge Proof of Solvency Blockchain

Welcome to the Zero-Knowledge Proof of Solvency Blockchain project! This project showcases a blockchain designed to prove the balance of a marketplace using zero-knowledge proofs. It simulates user deposits, transfers between clients, and provides proof of liabilities with each new transaction.

## Features

- **Simulated Deposits and Transfers:** Mimics the deposits of users to a marketplace and the transfers between clients.
- **Proof of Liabilities:** Generates proof of liabilities with every new transaction, ensuring transparency and trust.
- **Proof of Inclusion:** Allows users to request proof of their inclusion, displaying their balance at every block from wallet creation to the present.
- **Multithreaded:** Runs a multithreaded server where one thread create blocks, and the other handle incoming TCP connections.
- **TCP Stream** The Client and the Server are connected with a TCP stream
- **Folding scheme:** Utilizes Circom for both proof of liabilities and proof of inclusion circuits, compiled with the nova folding scheme.
- **Distributed system:** The server creates the proofs, and the client requests and verifies them.

## Roadmap

- **Next Steps:** Go through the repo and work on the TODOs.
- **Future Work:** Stay tuned for more information on my proof of solvency. My master's thesis will be published soon!

## Server

### Start the Blockchain Node

- Start the blockchain node to begin generating blocks:
  ```sh
  cargo run start-node
  ```

## Client

- Open a separate terminal once the server is running.

### Fund a Wallet

- Fund a wallet with a specified address and an amount:
  ```
  cargo run fund-account  <address> <amount>
  ```

### Transfer Funds

- Transfer funds between wallets::
  ```
  cargo run transfer <from> <to> <amount>
  ```

### Get User Balance

- Retrieve the balance of a user:
  ```
  cargo run balance <address>
  ```

### Verify Proof of liabilities

- Request the proof of liabilities for the latest block and verifies it:
  ```
  cargo run verify
  ```

### Get User Balance History

- Request a proof of the user's balance, verify it and publish the verified data:
  ```
  cargo run balance-history <address>
  ```
