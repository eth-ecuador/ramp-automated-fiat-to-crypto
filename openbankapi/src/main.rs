mod error;
mod types;
mod contract;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tower_http::cors::{Any, CorsLayer};
use uuid::Uuid;
use dotenv::dotenv;

use crate::error::OpenBankError;
use crate::types::*;
use crate::contract::ContractClient;

// App state
#[derive(Clone)]
pub struct AppState {
    pub users: Arc<RwLock<HashMap<String, User>>>,
    pub accounts: Arc<RwLock<HashMap<String, Account>>>,
    pub transactions: Arc<RwLock<HashMap<String, Vec<Transaction>>>>,
    pub contract_client: Option<Arc<ContractClient>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            users: Arc::new(RwLock::new(HashMap::new())),
            accounts: Arc::new(RwLock::new(HashMap::new())),
            transactions: Arc::new(RwLock::new(HashMap::new())),
            contract_client: None,
        }
    }
    
    pub async fn with_contract(mut self) -> Result<Self, OpenBankError> {
        dotenv().ok();
        
        let contract_config = SmartContractConfig {
            contract_address: std::env::var("CONTRACT_ADDRESS")
                .map_err(|_| OpenBankError::SmartContractError { 
                    message: "CONTRACT_ADDRESS not found in .env file".to_string() 
                })?,
            owner_private_key: std::env::var("OWNER_PRIVATE_KEY")
                .map_err(|_| OpenBankError::SmartContractError { 
                    message: "OWNER_PRIVATE_KEY not found in .env file".to_string() 
                })?,
            rpc_url: std::env::var("RPC_URL")
                .unwrap_or_else(|_| "http://localhost:8545".to_string()),
            chain_id: std::env::var("CHAIN_ID")
                .unwrap_or_else(|_| "31337".to_string())
                .parse()
                .unwrap_or(31337),
        };
        
        let contract_client = ContractClient::new(contract_config).await?;
        self.contract_client = Some(Arc::new(contract_client));
        
        Ok(self)
    }
}

// API handlers
async fn create_user(
    State(state): State<AppState>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<(StatusCode, Json<ApiResponse<User>>), (StatusCode, Json<OpenBankError>)> {
    let user_id = Uuid::new_v4().to_string();
    
    // Validate wallet address if provided
    if let Some(ref wallet_address) = payload.wallet_address {
        // Basic Ethereum address validation
        if !wallet_address.starts_with("0x") || wallet_address.len() != 42 {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(OpenBankError::InvalidWalletAddress { address: wallet_address.clone() }),
            ));
        }
    }
    
    // Check if user already exists (by email)
    {
        let users = state.users.read().unwrap();
        if users.values().any(|u| u.email == payload.email) {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(OpenBankError::UserAlreadyExists { email: payload.email }),
            ));
        }
        
        // Check if wallet address is already associated with another user
        if let Some(ref wallet_address) = payload.wallet_address {
            if users.values().any(|u| u.wallet_address.as_ref() == Some(wallet_address)) {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(OpenBankError::InvalidWalletAddress { address: wallet_address.clone() }),
                ));
            }
        }
    }
    
    let user = User {
        id: user_id.clone(),
        email: payload.email,
        name: payload.name,
        wallet_address: payload.wallet_address,
        created_at: chrono::Utc::now(),
        accounts: Vec::new(),
    };
    
    {
        let mut users = state.users.write().unwrap();
        users.insert(user_id.clone(), user.clone());
    }
    
    // If wallet address is provided, try to get balance from smart contract
    if let Some(ref wallet_address) = user.wallet_address {
        if let Some(ref contract_client) = state.contract_client {
            match contract_client.get_user_balance(wallet_address.clone()).await {
                Ok(balance) => {
                    println!("User {} has contract balance: deposited={}, withdrawn={}", 
                        user.email, balance.deposited, balance.withdrawn);
                }
                Err(e) => {
                    println!("Warning: Could not get contract balance for {}: {:?}", wallet_address, e);
                }
            }
        }
    }
    
    Ok((StatusCode::OK, Json(ApiResponse {
        success: true,
        data: Some(user),
        error: None,
    })))
}

async fn get_user(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
) -> Result<(StatusCode, Json<ApiResponse<User>>), (StatusCode, Json<OpenBankError>)> {
    let users = state.users.read().unwrap();
    
    match users.get(&user_id) {
        Some(user) => Ok((StatusCode::OK, Json(ApiResponse {
            success: true,
            data: Some(user.clone()),
            error: None,
        }))),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(OpenBankError::UserNotFound { user_id }),
        )),
    }
}

async fn create_account(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
    Json(payload): Json<CreateAccountRequest>,
) -> Result<(StatusCode, Json<ApiResponse<Account>>), (StatusCode, Json<OpenBankError>)> {
    // Validate user exists
    {
        let users = state.users.read().unwrap();
        if !users.contains_key(&user_id) {
            return Err((
                StatusCode::NOT_FOUND,
                Json(OpenBankError::UserNotFound { user_id: user_id.clone() }),
            ));
        }
    }
    
    // Always create a deposit tracking account
    let account_type = AccountType::Deposit;
    
    let account_id = Uuid::new_v4().to_string();
    let account = Account {
        id: account_id.clone(),
        user_id: user_id.clone(),
        account_type,
        balance: 0.0,
        currency: payload.currency,
        created_at: chrono::Utc::now(),
        is_active: true,
    };
    
    // Add account to state
    {
        let mut accounts = state.accounts.write().unwrap();
        accounts.insert(account_id.clone(), account.clone());
    }
    
    // Add account to user
    {
        let mut users = state.users.write().unwrap();
        if let Some(user) = users.get_mut(&user_id) {
            user.accounts.push(account_id.clone());
        }
    }
    
    // Initialize transactions list
    {
        let mut transactions = state.transactions.write().unwrap();
        transactions.insert(account_id.clone(), Vec::new());
    }
    
    Ok((StatusCode::OK, Json(ApiResponse {
        success: true,
        data: Some(account),
        error: None,
    })))
}

async fn get_account(
    State(state): State<AppState>,
    Path(account_id): Path<String>,
) -> Result<(StatusCode, Json<ApiResponse<Account>>), (StatusCode, Json<OpenBankError>)> {
    let accounts = state.accounts.read().unwrap();
    
    match accounts.get(&account_id) {
        Some(account) => Ok((StatusCode::OK, Json(ApiResponse {
            success: true,
            data: Some(account.clone()),
            error: None,
        }))),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(OpenBankError::AccountNotFound { account_id }),
        )),
    }
}

async fn deposit(
    State(state): State<AppState>,
    Path(account_id): Path<String>,
    Json(payload): Json<DepositRequest>,
) -> Result<(StatusCode, Json<ApiResponse<Transaction>>), (StatusCode, Json<OpenBankError>)> {
    if payload.amount <= 0.0 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(OpenBankError::InvalidAmount { amount: payload.amount }),
        ));
    }
    
    let transaction_id = Uuid::new_v4().to_string();
    
    // Update account balance and get user_id
    let (_balance_after, user_id) = {
        let mut accounts = state.accounts.write().unwrap();
        match accounts.get_mut(&account_id) {
            Some(account) => {
                account.balance += payload.amount;
                (account.balance, account.user_id.clone())
            }
            None => {
                return Err((
                    StatusCode::NOT_FOUND,
                    Json(OpenBankError::AccountNotFound { account_id }),
                ));
            }
        }
    };
    
    // Create transaction record
    let transaction = Transaction {
        id: transaction_id.clone(),
        user_id,
        account_id: account_id.clone(),
        amount: payload.amount,
        transaction_type: TransactionType::Deposit,
        description: payload.description.unwrap_or_else(|| "Deposit".to_string()),
        timestamp: chrono::Utc::now(),
        balance_after: _balance_after,
    };
    
    // Add transaction to history
    {
        let mut transactions = state.transactions.write().unwrap();
        if let Some(account_transactions) = transactions.get_mut(&account_id) {
            account_transactions.push(transaction.clone());
        }
    }
    
    Ok((StatusCode::OK, Json(ApiResponse {
        success: true,
        data: Some(transaction),
        error: None,
    })))
}

async fn get_transactions(
    State(state): State<AppState>,
    Path(account_id): Path<String>,
) -> Result<(StatusCode, Json<ApiResponse<Vec<Transaction>>>), (StatusCode, Json<OpenBankError>)> {
    let transactions = state.transactions.read().unwrap();
    
    match transactions.get(&account_id) {
        Some(account_transactions) => Ok((StatusCode::OK, Json(ApiResponse {
            success: true,
            data: Some(account_transactions.clone()),
            error: None,
        }))),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(OpenBankError::AccountNotFound { account_id }),
        )),
    }
}

async fn get_user_accounts(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
) -> Result<(StatusCode, Json<ApiResponse<Vec<Account>>>), (StatusCode, Json<OpenBankError>)> {
    let users = state.users.read().unwrap();
    let accounts = state.accounts.read().unwrap();
    
    match users.get(&user_id) {
        Some(user) => {
            let user_accounts: Vec<Account> = user.accounts
                .iter()
                .filter_map(|account_id| accounts.get(account_id).cloned())
                .collect();
            
            Ok((StatusCode::OK, Json(ApiResponse {
                success: true,
                data: Some(user_accounts),
                error: None,
            })))
        }
        None => Err((
            StatusCode::NOT_FOUND,
            Json(OpenBankError::UserNotFound { user_id }),
        )),
    }
}

#[axum::debug_handler]
async fn withdraw_to_wallet(
    State(state): State<AppState>,
    Json(payload): Json<WithdrawRequest>,
) -> Result<(StatusCode, Json<ApiResponse<String>>), (StatusCode, Json<OpenBankError>)> {
    // Get user to check if they have a wallet address
    let wallet_address = {
        let users = state.users.read().unwrap();
        let user = users.get(&payload.user_id)
            .ok_or_else(|| (
                StatusCode::NOT_FOUND,
                Json(OpenBankError::UserNotFound { user_id: payload.user_id.clone() })
            ))?;
        
        user.wallet_address.as_ref()
            .ok_or_else(|| (
                StatusCode::BAD_REQUEST,
                Json(OpenBankError::NoWalletAddress)
            ))?
            .clone()
    };
    
    // Validate amount
    if payload.amount <= 0.0 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(OpenBankError::InvalidAmount { amount: payload.amount }),
        ));
    }
    
    // Convert amount to USDT smallest unit (6 decimals)
    let amount_usdt = (payload.amount * 1_000_000.0) as u64;
    
    // Send transaction to smart contract
    if let Some(ref contract_client) = state.contract_client {
        let description = payload.description.unwrap_or_else(|| "API withdrawal".to_string());
        
        contract_client.send_usdt_to_address(
            wallet_address.clone(),
            amount_usdt,
            description
        ).await.map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(e)
        ))?;
        
        Ok((StatusCode::OK, Json(ApiResponse {
            success: true,
            data: Some(format!("Successfully sent {} USDT to {}", payload.amount, wallet_address)),
            error: None,
        })))
    } else {
        Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(OpenBankError::SmartContractError { 
                message: "Smart contract client not configured".to_string() 
            })
        ))
    }
}

async fn health_check() -> Json<ApiResponse<&'static str>> {
    Json(ApiResponse {
        success: true,
        data: Some("OpenBank API is running!"),
        error: None,
    })
}

#[tokio::main]
async fn main() {
    println!("OnrampTee & OpenBank Mock API...");
    
    let state = AppState::new();
    
    // Initialize contract client (REQUIRED - API won't work without it)
    let state = state.with_contract().await.expect("Failed to initialize smart contract integration. Please check your .env file with CONTRACT_ADDRESS, OWNER_PRIVATE_KEY, RPC_URL, and CHAIN_ID");
    println!("Smart contract integration enabled!");
    
    // Configure CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);
    
    // Build router
    let app = Router::new()
        //Openbank API mocking
        .route("/health", get(health_check))
        .route("/users", post(create_user))
        .route("/users/:user_id", get(get_user))
        .route("/users/:user_id/accounts", get(get_user_accounts))
        .route("/users/register/:user_id", post(create_account))
        .route("/accounts/:account_id", get(get_account))
        .route("/accounts/:account_id/deposit", post(deposit))
        .route("/accounts/:account_id/transactions", get(get_transactions))
        .route("/withdraw", post(withdraw_to_wallet))
        
        //OnrampTee routes
        
        .layer(cors)
        .with_state(state);
    
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
    println!("RampTee running on http://127.0.0.1:3000");
    println!("Available endpoints:");
    println!("   GET  /health - Health check");
    println!("   POST /users - Create user");
    println!("   GET  /users/:user_id - Get user");
    println!("   GET  /users/:user_id/accounts - Get user accounts");
    println!("   POST /users/register/:user_id - Create account");
    println!("   GET  /accounts/:account_id - Get account");
    println!("   POST /accounts/:account_id/deposit - Deposit money");
    println!("   GET  /accounts/:account_id/transactions - Get transaction history");
    println!("   POST /withdraw - Withdraw USDT to user wallet (owner only)");
    
    axum::serve(listener, app).await.unwrap();
}
