// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "forge-std/Test.sol";
import "../src/USDTToken.sol";
import "../src/OnrampEcuador.sol";

contract OnrampTeeTest is Test {
    USDTToken public usdtToken;
    OnrampEcuador public onrampEcuador;
    
    address public owner = address(1);
    address public user1 = address(2);
    address public user2 = address(3);
    
    uint256 public constant INITIAL_SUPPLY = 1000000 * 10**6; // 1,000,000 USDT
    uint256 public constant DEPOSIT_AMOUNT = 1000 * 10**6; // 1,000 USDT
    uint256 public constant WITHDRAW_AMOUNT = 500 * 10**6; // 500 USDT
    
    function setUp() public {
        vm.startPrank(owner);
        
        // Deploy contracts
        usdtToken = new USDTToken();
        onrampEcuador = new OnrampEcuador();
        
        // Set USDT token in OnrampEcuador
        onrampEcuador.setUSDTToken(address(usdtToken));
        
        vm.stopPrank();
    }
    
    function test_InitialSetup() public {
        // Check USDT token setup
        assertEq(usdtToken.balanceOf(owner), INITIAL_SUPPLY);
        assertEq(usdtToken.decimals(), 6);
        assertEq(usdtToken.name(), "Tether USD");
        assertEq(usdtToken.symbol(), "USDT");
        
        // Check OnrampEcuador setup
        assertEq(address(onrampEcuador.usdtToken()), address(usdtToken));
        assertEq(onrampEcuador.owner(), owner);
    }
    
    function test_MintTokens() public {
        vm.startPrank(owner);
        
        uint256 mintAmount = 10000 * 10**6; // 10,000 USDT
        usdtToken.mint(user1, mintAmount);
        
        assertEq(usdtToken.balanceOf(user1), mintAmount);
        
        vm.stopPrank();
    }
    
    function test_DepositUSDT() public {
        // Mint tokens to user1
        vm.startPrank(owner);
        usdtToken.mint(user1, DEPOSIT_AMOUNT);
        vm.stopPrank();
        
        // User1 approves and deposits
        vm.startPrank(user1);
        usdtToken.approve(address(onrampEcuador), DEPOSIT_AMOUNT);
        onrampEcuador.depositUSDT(DEPOSIT_AMOUNT, "Test deposit");
        vm.stopPrank();
        
        // Check balances
        assertEq(usdtToken.balanceOf(user1), 0);
        assertEq(usdtToken.balanceOf(address(onrampEcuador)), DEPOSIT_AMOUNT);
        
        // Check user balance
        OnrampEcuador.UserBalance memory balance = onrampEcuador.getUserBalance(user1);
        assertEq(balance.deposited, DEPOSIT_AMOUNT);
        assertEq(balance.withdrawn, 0);
        assertEq(balance.hasDeposited, true);
        
        // Check contract stats
        (uint256 totalUsers, uint256 totalDeposits, uint256 totalWithdrawals, uint256 contractBalance, uint256 totalTransactions) = onrampEcuador.getContractStats();
        assertEq(totalUsers, 1);
        assertEq(totalDeposits, DEPOSIT_AMOUNT);
        assertEq(totalWithdrawals, 0);
        assertEq(contractBalance, DEPOSIT_AMOUNT);
        assertEq(totalTransactions, 1);
    }
    
    function test_UsersCannotWithdraw() public {
        // Setup: Mint and deposit
        vm.startPrank(owner);
        usdtToken.mint(user1, DEPOSIT_AMOUNT);
        vm.stopPrank();
        
        vm.startPrank(user1);
        usdtToken.approve(address(onrampEcuador), DEPOSIT_AMOUNT);
        onrampEcuador.depositUSDT(DEPOSIT_AMOUNT, "Test deposit");
        
        // Try to withdraw (should fail - function doesn't exist)
        // onrampEcuador.withdrawUSDT(WITHDRAW_AMOUNT, "Test withdrawal"); // This line would cause compilation error
        
        vm.stopPrank();
        
        // Check that user still has no tokens (only deposited)
        assertEq(usdtToken.balanceOf(user1), 0);
        assertEq(usdtToken.balanceOf(address(onrampEcuador)), DEPOSIT_AMOUNT);
        
        // Check user balance shows only deposit
        OnrampEcuador.UserBalance memory balance = onrampEcuador.getUserBalance(user1);
        assertEq(balance.deposited, DEPOSIT_AMOUNT);
        assertEq(balance.withdrawn, 0);
    }
    
    function test_DepositAndWithdrawSpecificAccount(address testAccount) public {
        // Skip if testAccount is zero address or owner
        vm.assume(testAccount != address(0) && testAccount != owner);
        
        uint256 depositAmount = 2000 * 10**6; // 2,000 USDT
        uint256 withdrawAmount = 800 * 10**6;  // 800 USDT
        
        // Mint tokens to the specific account
        vm.startPrank(owner);
        usdtToken.mint(testAccount, depositAmount);
        vm.stopPrank();
        
        // Verify initial balance
        assertEq(usdtToken.balanceOf(testAccount), depositAmount);
        
        // TestAccount approves and deposits
        vm.startPrank(testAccount);
        usdtToken.approve(address(onrampEcuador), depositAmount);
        onrampEcuador.depositUSDT(depositAmount, "Specific account deposit");
        vm.stopPrank();
        
        // Check deposit was successful
        assertEq(usdtToken.balanceOf(testAccount), 0);
        assertEq(usdtToken.balanceOf(address(onrampEcuador)), depositAmount);
        
        // Check user balance after deposit
        OnrampEcuador.UserBalance memory balanceAfterDeposit = onrampEcuador.getUserBalance(testAccount);
        assertEq(balanceAfterDeposit.deposited, depositAmount);
        assertEq(balanceAfterDeposit.withdrawn, 0);
        assertEq(balanceAfterDeposit.hasDeposited, true);
        
        // Owner sends USDT to testAccount
        vm.startPrank(owner);
        onrampEcuador.sendUSDTToAddress(testAccount, withdrawAmount, "Specific account withdrawal");
        vm.stopPrank();
        
        // Check withdrawal was successful
        assertEq(usdtToken.balanceOf(testAccount), withdrawAmount);
        assertEq(usdtToken.balanceOf(address(onrampEcuador)), depositAmount - withdrawAmount);
        
        // Check user balance after withdrawal
        OnrampEcuador.UserBalance memory balanceAfterWithdrawal = onrampEcuador.getUserBalance(testAccount);
        assertEq(balanceAfterWithdrawal.deposited, depositAmount);
        assertEq(balanceAfterWithdrawal.withdrawn, withdrawAmount);
        
        // Check available balance
        assertEq(onrampEcuador.getAvailableBalance(testAccount), depositAmount - withdrawAmount);
        
        // Check transaction history
        uint256[] memory userTransactions = onrampEcuador.getUserTransactions(testAccount, 10);
        assertEq(userTransactions.length, 2);
        
        // Verify first transaction (deposit)
        OnrampEcuador.Transaction memory tx1 = onrampEcuador.getTransaction(userTransactions[0]);
        assertEq(tx1.user, testAccount);
        assertEq(tx1.amount, depositAmount);
        assertEq(tx1.isDeposit, true);
        assertEq(tx1.description, "Specific account deposit");
        
        // Verify second transaction (withdrawal)
        OnrampEcuador.Transaction memory tx2 = onrampEcuador.getTransaction(userTransactions[1]);
        assertEq(tx2.user, testAccount);
        assertEq(tx2.amount, withdrawAmount);
        assertEq(tx2.isDeposit, false);
        assertEq(tx2.description, "Specific account withdrawal");
    }
    
    function test_WithdrawMoreThanDeposited() public {
        // Setup: Mint and deposit
        vm.startPrank(owner);
        usdtToken.mint(user1, DEPOSIT_AMOUNT);
        vm.stopPrank();
        
        vm.startPrank(user1);
        usdtToken.approve(address(onrampEcuador), DEPOSIT_AMOUNT);
        onrampEcuador.depositUSDT(DEPOSIT_AMOUNT, "Test deposit");
        
        // Try to send more than deposited (as owner)
        uint256 excessiveAmount = DEPOSIT_AMOUNT + 1000 * 10**6;
        vm.expectRevert("Insufficient contract balance");
        onrampEcuador.sendUSDTToAddress(user1, excessiveAmount, "Test withdrawal");
        vm.stopPrank();
    }
    
    function test_MultipleUsers() public {
        // Setup: Mint tokens to both users
        vm.startPrank(owner);
        usdtToken.mint(user1, DEPOSIT_AMOUNT);
        usdtToken.mint(user2, DEPOSIT_AMOUNT);
        vm.stopPrank();
        
        // User1 deposits
        vm.startPrank(user1);
        usdtToken.approve(address(onrampEcuador), DEPOSIT_AMOUNT);
        onrampEcuador.depositUSDT(DEPOSIT_AMOUNT, "User1 deposit");
        vm.stopPrank();
        
        // User2 deposits
        vm.startPrank(user2);
        usdtToken.approve(address(onrampEcuador), DEPOSIT_AMOUNT);
        onrampEcuador.depositUSDT(DEPOSIT_AMOUNT, "User2 deposit");
        vm.stopPrank();
        
        // Check contract stats
        (uint256 totalUsers, uint256 totalDeposits, uint256 totalWithdrawals, uint256 contractBalance, uint256 totalTransactions) = onrampEcuador.getContractStats();
        assertEq(totalUsers, 2);
        assertEq(totalDeposits, DEPOSIT_AMOUNT * 2);
        assertEq(contractBalance, DEPOSIT_AMOUNT * 2);
        assertEq(totalTransactions, 2);
    }
    
    function test_TransactionHistory() public {
        // Setup: Mint and deposit
        vm.startPrank(owner);
        usdtToken.mint(user1, DEPOSIT_AMOUNT);
        vm.stopPrank();
        
        vm.startPrank(user1);
        usdtToken.approve(address(onrampEcuador), DEPOSIT_AMOUNT);
        onrampEcuador.depositUSDT(DEPOSIT_AMOUNT, "Test deposit");
        vm.stopPrank();
        
        // Owner sends tokens to user1
        vm.startPrank(owner);
        onrampEcuador.sendUSDTToAddress(user1, WITHDRAW_AMOUNT, "Owner payment");
        vm.stopPrank();
        
        // Check transaction history
        uint256[] memory userTransactions = onrampEcuador.getUserTransactions(user1, 10);
        assertEq(userTransactions.length, 2);
        
        // Check first transaction (deposit)
        OnrampEcuador.Transaction memory tx1 = onrampEcuador.getTransaction(userTransactions[0]);
        assertEq(tx1.user, user1);
        assertEq(tx1.amount, DEPOSIT_AMOUNT);
        assertEq(tx1.isDeposit, true);
        assertEq(tx1.description, "Test deposit");
        
        // Check second transaction (owner payment)
        OnrampEcuador.Transaction memory tx2 = onrampEcuador.getTransaction(userTransactions[1]);
        assertEq(tx2.user, user1);
        assertEq(tx2.amount, WITHDRAW_AMOUNT);
        assertEq(tx2.isDeposit, false);
        assertEq(tx2.description, "Owner payment");
    }
    
    function test_PauseAndUnpause() public {
        vm.startPrank(owner);
        
        // Pause contract
        onrampEcuador.pause();
        assertTrue(onrampEcuador.paused());
        
        // Unpause contract
        onrampEcuador.unpause();
        assertFalse(onrampEcuador.paused());
        
        vm.stopPrank();
    }
    
    function test_DepositWhenPaused() public {
        // Setup: Mint tokens to user1
        vm.startPrank(owner);
        usdtToken.mint(user1, DEPOSIT_AMOUNT);
        onrampEcuador.pause();
        vm.stopPrank();
        
        // Try to deposit when paused
        vm.startPrank(user1);
        usdtToken.approve(address(onrampEcuador), DEPOSIT_AMOUNT);
        vm.expectRevert();
        onrampEcuador.depositUSDT(DEPOSIT_AMOUNT, "Test deposit");
        vm.stopPrank();
    }
    
    function test_EmergencyWithdraw() public {
        // Setup: Mint and deposit
        vm.startPrank(owner);
        usdtToken.mint(user1, DEPOSIT_AMOUNT);
        vm.stopPrank();
        
        vm.startPrank(user1);
        usdtToken.approve(address(onrampEcuador), DEPOSIT_AMOUNT);
        onrampEcuador.depositUSDT(DEPOSIT_AMOUNT, "Test deposit");
        vm.stopPrank();
        
        // Emergency withdraw
        vm.startPrank(owner);
        onrampEcuador.emergencyWithdraw();
        vm.stopPrank();
        
        // Check balances
        assertEq(usdtToken.balanceOf(address(onrampEcuador)), 0);
        assertEq(usdtToken.balanceOf(owner), INITIAL_SUPPLY + DEPOSIT_AMOUNT);
    }
    
    function test_NonOwnerCannotMint() public {
        vm.startPrank(user1);
        vm.expectRevert();
        usdtToken.mint(user2, 1000 * 10**6);
        vm.stopPrank();
    }
    
    function test_NonOwnerCannotPause() public {
        vm.startPrank(user1);
        vm.expectRevert();
        onrampEcuador.pause();
        vm.stopPrank();
    }
    
    function test_NonOwnerCannotEmergencyWithdraw() public {
        vm.startPrank(user1);
        vm.expectRevert();
        onrampEcuador.emergencyWithdraw();
        vm.stopPrank();
    }
    
    function test_OwnerSendUSDTToAnyAddress() public {
        // Setup: Mint tokens to contract via deposit
        vm.startPrank(owner);
        usdtToken.mint(user1, DEPOSIT_AMOUNT);
        vm.stopPrank();
        
        vm.startPrank(user1);
        usdtToken.approve(address(onrampEcuador), DEPOSIT_AMOUNT);
        onrampEcuador.depositUSDT(DEPOSIT_AMOUNT, "Initial deposit");
        vm.stopPrank();
        
        // Owner sends USDT to user2
        uint256 sendAmount = 300 * 10**6; // 300 USDT
        vm.startPrank(owner);
        onrampEcuador.sendUSDTToAddress(user2, sendAmount, "Owner transfer to user2");
        vm.stopPrank();
        
        // Check balances
        assertEq(usdtToken.balanceOf(user2), sendAmount);
        assertEq(usdtToken.balanceOf(address(onrampEcuador)), DEPOSIT_AMOUNT - sendAmount);
        
        // Check transaction history for user2
        uint256[] memory user2Transactions = onrampEcuador.getUserTransactions(user2, 10);
        assertEq(user2Transactions.length, 1);
        
        OnrampEcuador.Transaction memory tx = onrampEcuador.getTransaction(user2Transactions[0]);
        assertEq(tx.user, user2);
        assertEq(tx.amount, sendAmount);
        assertEq(tx.isDeposit, false);
        assertEq(tx.description, "Owner transfer to user2");
    }
    
    function test_NonOwnerCannotSendUSDT() public {
        // Setup: Contract has some USDT
        vm.startPrank(owner);
        usdtToken.mint(user1, DEPOSIT_AMOUNT);
        vm.stopPrank();
        
        vm.startPrank(user1);
        usdtToken.approve(address(onrampEcuador), DEPOSIT_AMOUNT);
        onrampEcuador.depositUSDT(DEPOSIT_AMOUNT, "Initial deposit");
        vm.stopPrank();
        
        // User1 tries to send USDT (should fail)
        vm.startPrank(user1);
        vm.expectRevert();
        onrampEcuador.sendUSDTToAddress(user2, 100 * 10**6, "Unauthorized transfer");
        vm.stopPrank();
    }
    
    function test_OwnerSendUSDTToMultipleAddresses() public {
        // Setup: Contract has USDT
        vm.startPrank(owner);
        usdtToken.mint(user1, DEPOSIT_AMOUNT * 2);
        vm.stopPrank();
        
        vm.startPrank(user1);
        usdtToken.approve(address(onrampEcuador), DEPOSIT_AMOUNT * 2);
        onrampEcuador.depositUSDT(DEPOSIT_AMOUNT * 2, "Large deposit");
        vm.stopPrank();
        
        // Owner sends to multiple addresses
        address[] memory recipients = new address[](3);
        recipients[0] = address(0x1111);
        recipients[1] = address(0x2222);
        recipients[2] = address(0x3333);
        
        uint256 sendAmount = 200 * 10**6; // 200 USDT each
        
        vm.startPrank(owner);
        for (uint i = 0; i < recipients.length; i++) {
            onrampEcuador.sendUSDTToAddress(recipients[i], sendAmount, "Owner distribution");
        }
        vm.stopPrank();
        
        // Check all recipients received tokens
        for (uint i = 0; i < recipients.length; i++) {
            assertEq(usdtToken.balanceOf(recipients[i]), sendAmount);
        }
        
        // Check contract balance
        assertEq(usdtToken.balanceOf(address(onrampEcuador)), DEPOSIT_AMOUNT * 2 - (sendAmount * 3));
    }
}
