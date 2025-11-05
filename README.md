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
```sh
cargo run fund-account  <address> <amount>
```

### Transfer Funds

- Transfer funds between wallets::
```sh
cargo run transfer <from> <to> <amount>
```

### Get User Balance

- Retrieve the balance of a user:
```sh
cargo run balance <address>
```

### Verify Proof of liabilities

- Request the proof of liabilities for the latest block and verifies it:
```sh
cargo run verify
```

### Get User Balance History

- Request a proof of the user's balance, verify it and publish the verified data:
```sh
cargo run balance-history <address>
```

## Testing

Unit tests:
```sh
cargo test --lib
```

Integration tests:
```sh
cargo test --test proof_tests
```

Run all tests:
```sh
cargo test
```


### Future work
1. **Pre-compiled circuit library**: Compile circuits for tree depths 2-10, dynamically select based on user count: Integrate with [proof-of-solvency](https://github.com/AntoineCyr/proof_of_solvency)
2. **Persistent public parameters**: Cache PP generation to eliminate per-proof overhead (currently the main bottleneck). 
3. **Integrate log**: Better logging to find other bottlenecks
4. **Dynamic tree growth**: Automatically resize Merkle tree as users join
5. **Benchmarking suite**: Track proof times across circuit sizes
6. **Optimization**: Find ways to reduce proving and verifying time, like combining aggregation and folding to prove state update in parallel.