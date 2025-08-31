# OnrampTee - Decentralized Onramp Solution

> **Secure fiat-to-crypto onramp with Trusted Execution Environments (TEE)**

## Overview

OnrampTee enables seamless fiat-to-crypto conversion with secure smart contracts and a high-performance TEE-powered API. Deployed on multiple testnets with **Phala Network** as the trusted execution environment provider for enhanced security and privacy.

## Deployments

### Lisk Sepolia Testnet (Chain ID: 4202)
- **USDT Token**: [`0x7C2d1Cd423ffE6882C872890f649424144A526Af`](https://sepolia-blockscout.lisk.com/address/0x7C2d1Cd423ffE6882C872890f649424144A526Af)
- **OnrampEcuador**: [`0x1F5D59b32915c5498A062AA0b5E8D268d3127BC0`](https://sepolia-blockscout.lisk.com/address/0x1F5D59b32915c5498A062AA0b5E8D268d3127BC0)

### Ethereum Sepolia Testnet (Chain ID: 11155111)
- **OnrampEcuador**: [`0x1F5D59b32915c5498A062AA0b5E8D268d3127BC0`](https://sepolia.etherscan.io/address/0x1F5D59b32915c5498A062AA0b5E8D268d3127BC0#code)
- **USDT Token**: [`0x7C2d1Cd423ffE6882C872890f649424144A526Af`](https://sepolia.etherscan.io/address/0x7C2d1Cd423ffE6882C872890f649424144A526Af#code)

## API Endpoints

```bash
# Health Check
GET /health

# User Management
POST /users                    # Register new user
GET /users/{user_id}          # Get user details

# Account Operations
GET /accounts/{account_id}     # Get account info
POST /accounts/{account_id}/deposit    # Deposit funds
GET /accounts/{account_id}/transactions # Get transaction history

# Withdrawal
POST /withdraw                 # Withdraw to wallet
```

## Tech Stack

- **Smart Contracts**: Solidity 0.8.20 + OpenZeppelin
- **API**: Rust + Axum + Docker
- **TEE Provider**: Phala Network
- **Deployment**: Foundry + Multi-chain support

## Quick Start

```bash
# Deploy contracts
cd tokenlogic
forge script script/Deploy.s.sol:DeployScript --rpc-url https://rpc.sepolia-api.lisk.com/ --chain-id 4202 --broadcast

# Run API
cd openbankapi
docker build -t onramptee-api .
docker run -p 3000:3000 onramptee-api
```

## Contract Info

- **Token**: Tether USD (USDT) - 6 decimals
- **Supply**: 1,000,000 USDT
- **Features**: Reentrancy protection, Access control, Pausable

## Security Features

- **TEE Protection**: Phala Network provides hardware-level security
- **Smart Contract Security**: OpenZeppelin audited contracts
- **Multi-chain**: Distributed across multiple testnets

## Author

**protocolwhisper.eth**

---

