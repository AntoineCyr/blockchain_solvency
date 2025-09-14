# Lean Blockchain for Marketplace Solvency

This is a lean blockchain designed to serve as an internal process system for a marketplace. Its primary goal is to generate live proof of liabilities for every block, ensuring transparency and trust in the marketplace's financial operations.

## Overview

The system operates with a server-client architecture:

- **Server**: Maintains the blockchain, processes transactions, and generates zero-knowledge proofs for every block
- **Client**: Connects to the server to perform operations, request proofs, and verify them independently
- **Proof of Liabilities**: Generated for every block to prove the total liabilities of the marketplace
- **Proof of Inclusion**: Allows users to verify their complete balance history from wallet creation to present

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

## Limitations
Default is 4 user max (tree of 2 levels)
Recompile the circuit for more levels and more user.


## Current Status & Future Work

We currently have proof generation working for every block. Future improvements focus on reducing proof time:

1. Compile multiple circuits with a range of levels (for instance 2-10). We should use different circuits depending on our number of users.

2. Fix the public parameters generation. Currently we generate them for each proof.
