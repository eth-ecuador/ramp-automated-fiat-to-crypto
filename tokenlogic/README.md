# OnrampTee Smart Contracts (Foundry)

This directory contains the smart contracts for the OnrampTee project, built with Foundry for optimal performance and gas efficiency.

## Contracts Overview

### 1. USDTToken.sol
A custom ERC20 token that mimics USDT with 6 decimal places.

**Features:**
- Standard ERC20 functionality
- 6 decimal places (like real USDT)
- Minting capability (owner only)
- Burning capability
- Initial supply of 1,000,000 USDT

### 2. OnrampEcuador.sol
Main contract for handling USDT deposits and owner-controlled distributions.

**Features:**
- Deposit USDT tokens (anyone can deposit)
- Owner-controlled token distribution (only owner can send tokens)
- User balance tracking
- Transaction history
- Emergency pause functionality
- Statistics

## Quick Start

### Prerequisites
- [Foundry](https://getfoundry.sh/) installed
- Git

### Setup
```bash
# Clone dependencies
forge install

# Build contracts
forge build

# Run tests
forge test
```

### Deploy Contracts
```bash
# Set your private key
export PRIVATE_KEY=your_private_key_here

# Deploy to local network
forge script script/Deploy.s.sol --rpc-url http://localhost:8545 --broadcast

# Deploy to testnet
forge script script/Deploy.s.sol --rpc-url $SEPOLIA_RPC_URL --broadcast --verify
```

## Contract Functions

### USDTToken Functions

#### `mint(address to, uint256 amount)`
- **Purpose**: Mint new USDT tokens
- **Access**: Owner only
- **Parameters**: 
  - `to`: Recipient address
  - `amount`: Amount to mint (in smallest unit - 6 decimals)

#### `burn(uint256 amount)`
- **Purpose**: Burn tokens from caller
- **Access**: Anyone with sufficient balance
- **Parameters**: `amount`: Amount to burn

#### `getTokenInfo()`
- **Purpose**: Get token information
- **Returns**: Name, symbol, decimals, total supply

### OnrampEcuador Functions

#### `depositUSDT(uint256 amount, string description)`
- **Purpose**: Deposit USDT tokens to the contract
- **Parameters**:
  - `amount`: Amount to deposit (in smallest unit - 6 decimals)
  - `description`: Optional description



#### `getUserBalance(address userAddress)`
- **Purpose**: Get user's balance information
- **Returns**: UserBalance struct with deposit/withdrawal history

#### `getAvailableBalance(address userAddress)`
- **Purpose**: Get user's available balance for withdrawal
- **Returns**: Available amount for withdrawal

#### `getContractStats()`
- **Purpose**: Get contract statistics
- **Returns**: Total users, deposits, withdrawals, contract balance, transactions

#### `sendUSDTToAddress(address recipient, uint256 amount, string description)`
- **Purpose**: Send USDT to any address (owner only)
- **Access**: Owner only
- **Parameters**:
  - `recipient`: Address to send USDT to
  - `amount`: Amount to send (in smallest unit - 6 decimals)
  - `description`: Optional description for the transfer

## Usage Examples

### 1. Deploy Contracts
```bash
forge script script/Deploy.s.sol --rpc-url http://localhost:8545 --broadcast
```

### 2. Mint USDT to User
```solidity
// Mint 1000 USDT to user (1000 * 10^6 = 1,000,000,000)
await usdtToken.mint(userAddress, 1000000000);
```

### 3. Approve Spending
```solidity
// Approve OnrampEcuador to spend 1000 USDT
await usdtToken.approve(onrampEcuador.address, 1000000000);
```

### 4. Deposit USDT
```solidity
// Deposit 500 USDT
await onrampEcuador.depositUSDT(500000000, "Initial deposit");
```

### 5. Withdraw USDT
```solidity
// Withdraw 200 USDT (REMOVED - Users cannot withdraw)
// Only owner can send tokens to addresses
```

### 6. Owner Send USDT to Any Address
```solidity
// Owner sends 500 USDT to a specific address
await onrampEcuador.sendUSDTToAddress(recipientAddress, 500000000, "Owner transfer");
```

## Security Features

- **ReentrancyGuard**: Prevents reentrancy attacks
- **Pausable**: Emergency pause functionality
- **Ownable**: Owner-only functions
- **Input Validation**: Comprehensive parameter checks
- **Balance Checks**: Ensures sufficient funds before operations

## Events

### USDTToken Events
- `TokensMinted`: When tokens are minted
- `TokensBurned`: When tokens are burned

### OnrampEcuador Events
- `USDTTokenSet`: When USDT token is set
- `DepositMade`: When deposit is made
- `WithdrawalMade`: When withdrawal is made
- `EmergencyWithdraw`: When emergency withdrawal is executed

## Testing

Run the comprehensive test suite:
```bash
# Run all tests
forge test

# Run with verbose output
forge test -vv

# Run with gas reporting
forge test --gas-report

# Run specific test
forge test --match-test test_DepositUSDT
```

## Development

### Network Configuration
The contracts support multiple networks:
- **Anvil**: Local development (`http://localhost:8545`)
- **Sepolia**: Testnet
- **Mainnet**: Production deployment

### Environment Variables
```bash
PRIVATE_KEY=your_private_key
SEPOLIA_RPC_URL=your_sepolia_rpc_url
MAINNET_RPC_URL=your_mainnet_rpc_url
ETHERSCAN_API_KEY=your_etherscan_api_key
```

### Commands
```bash
# Build contracts
forge build

# Run tests
forge test

# Deploy
forge script script/Deploy.s.sol --rpc-url $RPC_URL --broadcast

# Verify on Etherscan
forge script script/Deploy.s.sol --rpc-url $RPC_URL --broadcast --verify

# Start local node
anvil

# Get contract size
forge build --sizes
```

## License

MIT License - see LICENSE file for details.

## Author

protocolwhisper.eth
