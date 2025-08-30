use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::error::OpenBankError;

// Data structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub email: String,
    pub name: String,
    pub wallet_address: Option<String>, // Ethereum wallet address
    pub created_at: DateTime<Utc>,
    pub accounts: Vec<String>, // Account IDs
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: String,
    pub user_id: String,
    pub account_type: AccountType,
    pub balance: f64,
    pub currency: String,
    pub created_at: DateTime<Utc>,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccountType {
    Deposit, // Account for tracking deposits
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: String,
    pub user_id: String,
    pub account_id: String,
    pub transaction_type: TransactionType,
    pub amount: f64,
    pub description: String,
    pub timestamp: DateTime<Utc>,
    pub balance_after: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionType {
    Deposit,
    Transfer,
}

// Smart Contract related types
#[derive(Debug, Clone)]
pub struct SmartContractConfig {
    pub contract_address: String,
    pub owner_private_key: String,
    pub rpc_url: String,
    pub chain_id: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractUserBalance {
    pub deposited: u64,
    pub withdrawn: u64,
    pub last_deposit: u64,
    pub last_withdrawal: u64,
    pub has_deposited: bool,
}

// Request/Response structures
#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub email: String,
    pub name: String,
    pub wallet_address: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateAccountRequest {
    pub currency: String, // e.g., "USD", "EUR", "GBP"
}

#[derive(Debug, Deserialize)]
pub struct DepositRequest {
    pub amount: f64,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct WithdrawRequest {
    pub user_id: String,
    pub amount: f64,
    pub description: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<OpenBankError>,
}
