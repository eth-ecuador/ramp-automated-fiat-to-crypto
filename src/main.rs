mod error;
mod types;

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

use crate::error::OpenBankError;
use crate::types::*;

// App state
#[derive(Clone)]
pub struct AppState {
    pub users: Arc<RwLock<HashMap<String, User>>>,
    pub accounts: Arc<RwLock<HashMap<String, Account>>>,
    pub transactions: Arc<RwLock<HashMap<String, Vec<Transaction>>>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            users: Arc::new(RwLock::new(HashMap::new())),
            accounts: Arc::new(RwLock::new(HashMap::new())),
            transactions: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

// API handlers
async fn create_user(
    State(state): State<AppState>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<(StatusCode, Json<ApiResponse<User>>), (StatusCode, Json<OpenBankError>)> {
    let user_id = Uuid::new_v4().to_string();
    
    // Check if user already exists
    {
        let users = state.users.read().unwrap();
        if users.values().any(|u| u.email == payload.email) {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(OpenBankError::UserAlreadyExists { email: payload.email }),
            ));
        }
    }
    
    let user = User {
        id: user_id.clone(),
        email: payload.email,
        name: payload.name,
        created_at: chrono::Utc::now(),
        accounts: Vec::new(),
    };
    
    {
        let mut users = state.users.write().unwrap();
        users.insert(user_id.clone(), user.clone());
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
    
    // Update account balance
    let balance_after = {
        let mut accounts = state.accounts.write().unwrap();
        match accounts.get_mut(&account_id) {
            Some(account) => {
                account.balance += payload.amount;
                account.balance
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
        account_id: account_id.clone(),
        transaction_type: TransactionType::Deposit,
        amount: payload.amount,
        description: payload.description.unwrap_or_else(|| "Deposit".to_string()),
        timestamp: chrono::Utc::now(),
        balance_after,
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
    
    axum::serve(listener, app).await.unwrap();
}
