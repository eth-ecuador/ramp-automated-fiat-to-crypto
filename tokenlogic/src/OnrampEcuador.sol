// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/utils/ReentrancyGuard.sol";
import "@openzeppelin/contracts/utils/Pausable.sol";

/**
 * @title OnrampEcuador
 * @dev Contract for depositing and withdrawing USDT tokens
 * @author protocolwhisper.eth
 */
contract OnrampEcuador is Ownable, ReentrancyGuard, Pausable {
    
    // USDT Token contract
    ERC20 public usdtToken;
    
    // User balance structure
    struct UserBalance {
        uint256 deposited;
        uint256 withdrawn;
        uint256 lastDeposit;
        uint256 lastWithdrawal;
        bool hasDeposited;
    }
    
    // Transaction structure
    struct Transaction {
        address user;
        uint256 amount;
        uint256 timestamp;
        bool isDeposit;
        string description;
    }
    
    // Events
    event USDTTokenSet(address indexed tokenAddress);
    event DepositMade(address indexed user, uint256 amount, string description, uint256 timestamp);
    event WithdrawalMade(address indexed user, uint256 amount, string description, uint256 timestamp);
    event EmergencyWithdraw(address indexed owner, uint256 amount, uint256 timestamp);
    
    // State variables
    mapping(address => UserBalance) public userBalances;
    mapping(uint256 => Transaction) public transactions;
    uint256 public transactionCounter;
    uint256 public totalDeposits;
    uint256 public totalWithdrawals;
    uint256 public totalUsers;
    
    // Modifiers
    modifier validAmount(uint256 amount) {
        require(amount > 0, "Amount must be greater than 0");
        _;
    }
    
    modifier validAddress(address addr) {
        require(addr != address(0), "Invalid address");
        _;
    }
    
    modifier tokenSet() {
        require(address(usdtToken) != address(0), "USDT token not set");
        _;
    }
    
    /**
     * @dev Constructor
     */
    constructor() Ownable(msg.sender) {
        transactionCounter = 0;
        totalDeposits = 0;
        totalWithdrawals = 0;
        totalUsers = 0;
    }
    
    /**
     * @dev Set the USDT token address (only owner)
     * @param tokenAddress The address of the USDT token contract
     */
    function setUSDTToken(address tokenAddress) external onlyOwner validAddress(tokenAddress) {
        usdtToken = ERC20(tokenAddress);
        emit USDTTokenSet(tokenAddress);
    }
    
    /**
     * @dev Deposit USDT tokens to the contract
     * @param amount Amount of USDT to deposit (in smallest unit - 6 decimals)
     * @param description Optional description for the deposit
     */
    function depositUSDT(uint256 amount, string memory description) 
        external 
        tokenSet 
        validAmount(amount) 
        nonReentrant 
        whenNotPaused 
    {
        require(usdtToken.balanceOf(msg.sender) >= amount, "Insufficient USDT balance");
        require(usdtToken.allowance(msg.sender, address(this)) >= amount, "Insufficient allowance");
        
        // Transfer USDT from user to contract
        require(usdtToken.transferFrom(msg.sender, address(this), amount), "Transfer failed");
        
        // Update user balance
        if (!userBalances[msg.sender].hasDeposited) {
            totalUsers++;
            userBalances[msg.sender].hasDeposited = true;
        }
        
        userBalances[msg.sender].deposited += amount;
        userBalances[msg.sender].lastDeposit = block.timestamp;
        
        // Update global stats
        totalDeposits += amount;
        
        // Record transaction
        transactions[transactionCounter] = Transaction({
            user: msg.sender,
            amount: amount,
            timestamp: block.timestamp,
            isDeposit: true,
            description: description
        });
        
        transactionCounter++;
        
        emit DepositMade(msg.sender, amount, description, block.timestamp);
    }
    
    /**
     * @dev Withdraw USDT tokens from the contract (REMOVED - Only owner can send tokens)
     * Users can only deposit, owner controls all withdrawals
     */
    
    /**
     * @dev Get user balance information
     * @param userAddress Address of the user
     * @return UserBalance struct with user's balance information
     */
    function getUserBalance(address userAddress) external view returns (UserBalance memory) {
        return userBalances[userAddress];
    }
    
    /**
     * @dev Get transaction information
     * @param transactionId ID of the transaction
     * @return Transaction struct with transaction details
     */
    function getTransaction(uint256 transactionId) external view returns (Transaction memory) {
        require(transactionId < transactionCounter, "Transaction does not exist");
        return transactions[transactionId];
    }
    
    /**
     * @dev Get contract statistics
     * @return _totalUsers Total number of users who have deposited
     * @return _totalDeposits Total amount deposited
     * @return _totalWithdrawals Total amount withdrawn
     * @return _contractBalance Current USDT balance of the contract
     * @return _totalTransactions Total number of transactions
     */
    function getContractStats() external view returns (
        uint256 _totalUsers,
        uint256 _totalDeposits,
        uint256 _totalWithdrawals,
        uint256 _contractBalance,
        uint256 _totalTransactions
    ) {
        return (
            totalUsers,
            totalDeposits,
            totalWithdrawals,
            usdtToken.balanceOf(address(this)),
            transactionCounter
        );
    }
    
    /**
     * @dev Get user's available balance for withdrawal
     * @param userAddress Address of the user
     * @return Available balance for withdrawal
     */
    function getAvailableBalance(address userAddress) external view returns (uint256) {
        UserBalance memory balance = userBalances[userAddress];
        if (balance.withdrawn >= balance.deposited) {
            return 0;
        }
        return balance.deposited - balance.withdrawn;
    }
    
    /**
     * @dev Emergency pause function (only owner)
     */
    function pause() external onlyOwner {
        _pause();
    }
    
    /**
     * @dev Unpause function (only owner)
     */
    function unpause() external onlyOwner {
        _unpause();
    }
    
    /**
     * @dev Emergency withdraw all USDT from contract (only owner)
     */
    function emergencyWithdraw() external onlyOwner tokenSet {
        uint256 balance = usdtToken.balanceOf(address(this));
        require(balance > 0, "No USDT to withdraw");
        require(usdtToken.transfer(owner(), balance), "Transfer failed");
        emit EmergencyWithdraw(owner(), balance, block.timestamp);
    }
    
    /**
     * @dev Send USDT to any address (only owner)
     * @param recipient Address to send USDT to
     * @param amount Amount of USDT to send (in smallest unit - 6 decimals)
     * @param description Optional description for the transfer
     */
    function sendUSDTToAddress(
        address recipient, 
        uint256 amount, 
        string memory description
    ) external onlyOwner tokenSet validAmount(amount) validAddress(recipient) {
        require(usdtToken.balanceOf(address(this)) >= amount, "Insufficient contract balance");
        
        // Transfer USDT from contract to recipient
        require(usdtToken.transfer(recipient, amount), "Transfer failed");
        
        // Update global stats
        totalWithdrawals += amount;
        
        // Record transaction
        transactions[transactionCounter] = Transaction({
            user: recipient,
            amount: amount,
            timestamp: block.timestamp,
            isDeposit: false,
            description: description
        });
        
        transactionCounter++;
        
        emit WithdrawalMade(recipient, amount, description, block.timestamp);
    }
    
    /**
     * @dev Get user's transaction history
     * @param userAddress Address of the user
     * @param limit Maximum number of transactions to return
     * @return Array of transaction IDs for the user
     */
    function getUserTransactions(address userAddress, uint256 limit) 
        external 
        view 
        returns (uint256[] memory) 
    {
        uint256[] memory userTxIds = new uint256[](limit);
        uint256 count = 0;
        
        for (uint256 i = 0; i < transactionCounter && count < limit; i++) {
            if (transactions[i].user == userAddress) {
                userTxIds[count] = i;
                count++;
            }
        }
        
        // Resize array to actual count
        uint256[] memory result = new uint256[](count);
        for (uint256 i = 0; i < count; i++) {
            result[i] = userTxIds[i];
        }
        
        return result;
    }
}
