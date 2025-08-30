use thiserror::Error;
use serde::Serialize;

#[derive(Error, Debug, Serialize)]
pub enum OpenBankError {
    #[error("User not found: {user_id}")]
    UserNotFound { user_id: String },
    
    #[error("Account not found: {account_id}")]
    AccountNotFound { account_id: String },
    
    #[error("Invalid amount: {amount}. Amount must be positive")]
    InvalidAmount { amount: f64 },
    
    #[error("User already exists: {email}")]
    UserAlreadyExists { email: String },
}
