// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "forge-std/Script.sol";
import "../src/USDTToken.sol";
import "../src/OnrampEcuador.sol";

contract DeployScript is Script {
    function run() external {
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");
        
        vm.startBroadcast(deployerPrivateKey);

        console.log("Starting deployment of OnrampTee contracts...");
        console.log("Deploying contracts with account:", vm.addr(deployerPrivateKey));

        // Deploy USDT Token
        console.log("\nDeploying USDTToken...");
        USDTToken usdtToken = new USDTToken();
        console.log("USDTToken deployed to:", address(usdtToken));

        // Get token info
        (string memory name, string memory symbol, uint8 decimals, uint256 totalSupply) = usdtToken.getTokenInfo();
        console.log("Token Info:");
        console.log("   Name:", name);
        console.log("   Symbol:", symbol);
        console.log("   Decimals:", decimals);
        console.log("   Total Supply:", totalSupply / 10**6, "USDT");

        // Deploy OnrampEcuador
        console.log("\nDeploying OnrampEcuador...");
        OnrampEcuador onrampEcuador = new OnrampEcuador();
        console.log("OnrampEcuador deployed to:", address(onrampEcuador));

        // Set USDT token in OnrampEcuador
        console.log("\nSetting USDT token in OnrampEcuador...");
        onrampEcuador.setUSDTToken(address(usdtToken));
        console.log("USDT token set successfully");

        // Get contract stats
        (uint256 totalUsers, uint256 totalDeposits, uint256 totalWithdrawals, uint256 contractBalance, uint256 totalTransactions) = onrampEcuador.getContractStats();
        console.log("\nContract Statistics:");
        console.log("   Total Users:", totalUsers);
        console.log("   Total Deposits:", totalDeposits / 10**6, "USDT");
        console.log("   Total Withdrawals:", totalWithdrawals / 10**6, "USDT");
        console.log("   Contract Balance:", contractBalance / 10**6, "USDT");
        console.log("   Total Transactions:", totalTransactions);

        console.log("\nDeployment completed successfully!");
        console.log("\nContract Addresses:");
        console.log("   USDTToken:", address(usdtToken));
        console.log("   OnrampEcuador:", address(onrampEcuador));
        console.log("   Deployer:", vm.addr(deployerPrivateKey));

        console.log("\nNext Steps:");
        console.log("   1. Mint USDT tokens to users: usdtToken.mint(address, amount)");
        console.log("   2. Approve USDT spending: usdtToken.approve(onrampEcuadorAddress, amount)");
        console.log("   3. Deposit USDT: onrampEcuador.depositUSDT(amount, description)");
        console.log("   4. Withdraw USDT: onrampEcuador.withdrawUSDT(amount, description)");

        vm.stopBroadcast();
    }
}
