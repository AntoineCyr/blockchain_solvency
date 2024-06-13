# Zero-Knowledge Proof of Solvency Blockchain

Welcome to the Zero-Knowledge Proof of Solvency Blockchain project! This project showcases a blockchain designed to prove the balance of a marketplace using zero-knowledge proofs. It simulates user deposits, transfers between clients, and provides proof of liabilities with each new transaction.

## Features

- **Simulated Deposits and Transfers:** Mimics the deposits of users to a marketplace and the transfers between clients.
- **Proof of Liabilities:** Generates proof of liabilities with every new transaction, ensuring transparency and trust.
- **Proof of Inclusion:** Allows users to request proof of their inclusion, displaying their balance at every block from wallet creation to the present.
- **Multithreaded:** Runs a multithreaded program with a server making blocks and a client handling transactions.
- **Circom Circuits:** Utilizes Circom for both proof of liabilities and proof of inclusion circuits, compiled with a novel folding scheme.

## Roadmap

- **Current:** Proofs are generated and verified by the server.
- **Next Steps:** Server will publish the proof, and clients will independently verify it.
- **Future Work:** Stay tuned for more information on my proof of solvency. My master's thesis will be published soon!

## Usage

### Start the Blockchain Node

- Start the blockchain node to begin generating blocks:
  ```sh
  cargo run start-node
  ```

### Create a Wallet

- Create a new wallet with a specified address and initial amount:
  ```
  cargo run create-account  <address> <amount>
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

### Get User Balance Proof (In Progress)

- Request a proof of the user's balance (feature in development):
  ```
  cargo run balance-proof <address>
  ```
