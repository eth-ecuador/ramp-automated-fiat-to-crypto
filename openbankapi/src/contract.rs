use ethers::{
    contract::{Contract, ContractInstance},
    core::types::{Address, U256},
    providers::{Http, Provider},
    signers::LocalWallet,
    abi::Abi,
    middleware::SignerMiddleware,
};
use std::sync::Arc;
use std::fs;
use crate::types::SmartContractConfig;
use crate::error::OpenBankError;

pub struct ContractClient {
    contract: ContractInstance<Arc<SignerMiddleware<Provider<Http>, LocalWallet>>, SignerMiddleware<Provider<Http>, LocalWallet>>,
    provider: Arc<Provider<Http>>,
}

impl ContractClient {
    pub async fn new(config: SmartContractConfig) -> Result<Self, OpenBankError> {
        let provider = Provider::<Http>::try_from(config.rpc_url)
            .map_err(|e| OpenBankError::SmartContractError { 
                message: format!("Failed to create provider: {}", e) 
            })?;
        
        let wallet = config.owner_private_key
            .parse::<LocalWallet>()
            .map_err(|e| OpenBankError::SmartContractError { 
                message: format!("Invalid private key: {}", e) 
            })?;
        
        let contract_address = config.contract_address
            .parse::<Address>()
            .map_err(|e| OpenBankError::SmartContractError { 
                message: format!("Invalid contract address: {}", e) 
            })?;
        
        // Read ABI from JSON file
        let abi_content = fs::read_to_string("OnrampEcuador.json")
            .map_err(|e| OpenBankError::SmartContractError { 
                message: format!("Failed to read ABI file: {}", e) 
            })?;
        
        let contract_json: serde_json::Value = serde_json::from_str(&abi_content)
            .map_err(|e| OpenBankError::SmartContractError { 
                message: format!("Failed to parse ABI JSON: {}", e) 
            })?;
        
        let abi = contract_json["abi"]
            .as_array()
            .ok_or_else(|| OpenBankError::SmartContractError { 
                message: "ABI not found in contract JSON".to_string() 
            })?;
        
        let abi_string = serde_json::to_string(abi)
            .map_err(|e| OpenBankError::SmartContractError { 
                message: format!("Failed to serialize ABI: {}", e) 
            })?;
        
        // Parse ABI
        let abi: Abi = serde_json::from_str(&abi_string)
            .map_err(|e| OpenBankError::SmartContractError { 
                message: format!("Failed to parse ABI: {}", e) 
            })?;
        
        // Create signer middleware
        let client = SignerMiddleware::new(provider.clone(), wallet);
        let client = Arc::new(client);
        
        // Create contract instance
        let contract = Contract::new(contract_address, abi, client.clone());
        
        let provider = Arc::new(provider);
        
        Ok(Self { contract, provider })
    }
    
    pub async fn deposit_usdt(&self, amount: u64, description: String) -> Result<(), OpenBankError> {
        let amount_wei = U256::from(amount);
        
        self.contract
            .method::<_, ()>("depositUSDT", (amount_wei, description))
            .map_err(|e| OpenBankError::SmartContractError { 
                message: format!("Failed to call depositUSDT: {}", e) 
            })?
            .send()
            .await
            .map_err(|e| OpenBankError::SmartContractError { 
                message: format!("Failed to send deposit transaction: {}", e) 
            })?;
        
        Ok(())
    }
    
    pub async fn send_usdt_to_address(
        &self, 
        recipient: String, 
        amount: u64, 
        description: String
    ) -> Result<(), OpenBankError> {
        let recipient = recipient
            .parse::<Address>()
            .map_err(|_e| OpenBankError::InvalidWalletAddress { address: recipient.clone() })?;
        
        let amount_wei = U256::from(amount);
        
        self.contract
            .method::<_, ()>("sendUSDTToAddress", (recipient, amount_wei, description))
            .map_err(|e| OpenBankError::SmartContractError { 
                message: format!("Failed to call sendUSDTToAddress: {}", e) 
            })?
            .send()
            .await
            .map_err(|e| OpenBankError::SmartContractError { 
                message: format!("Failed to send withdrawal transaction: {}", e) 
            })?;
        
        Ok(())
    }
    
    pub async fn get_user_balance(&self, user_address: String) -> Result<crate::types::ContractUserBalance, OpenBankError> {
        let user_address = user_address
            .parse::<Address>()
            .map_err(|_e| OpenBankError::InvalidWalletAddress { address: user_address.clone() })?;
        
        let result: (U256, U256, U256, U256, bool) = self.contract
            .method("getUserBalance", user_address)
            .map_err(|e| OpenBankError::SmartContractError { 
                message: format!("Failed to call getUserBalance: {}", e) 
            })?
            .call()
            .await
            .map_err(|e| OpenBankError::SmartContractError { 
                message: format!("Failed to get user balance: {}", e) 
            })?;
        
        Ok(crate::types::ContractUserBalance {
            deposited: result.0.as_u64(),
            withdrawn: result.1.as_u64(),
            last_deposit: result.2.as_u64(),
            last_withdrawal: result.3.as_u64(),
            has_deposited: result.4,
        })
    }
    
    pub async fn get_contract_stats(&self) -> Result<(u64, u64, u64, u64, u64), OpenBankError> {
        let result: (U256, U256, U256, U256, U256) = self.contract
            .method("getContractStats", ())
            .map_err(|e| OpenBankError::SmartContractError { 
                message: format!("Failed to call getContractStats: {}", e) 
            })?
            .call()
            .await
            .map_err(|e| OpenBankError::SmartContractError { 
                message: format!("Failed to get contract stats: {}", e) 
            })?;
        
        Ok((
            result.0.as_u64(),
            result.1.as_u64(),
            result.2.as_u64(),
            result.3.as_u64(),
            result.4.as_u64(),
        ))
    }
}
