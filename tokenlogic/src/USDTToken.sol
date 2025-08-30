// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import "@openzeppelin/contracts/access/Ownable.sol";

/**
 * @title USDTToken
 * @dev Custom USDT token for OnrampEcuador
 * @author protocolwhisper.eth
 */
contract USDTToken is ERC20, Ownable {
    
    uint8 private _decimals = 6; // USDT has 6 decimals
    
    // Events
    event TokensMinted(address indexed to, uint256 amount, uint256 timestamp);
    event TokensBurned(address indexed from, uint256 amount, uint256 timestamp);
    
    constructor() ERC20("Tether USD", "USDT") Ownable(msg.sender) {
        // Mint initial supply to owner (1,000,000 USDT)
        _mint(msg.sender, 1000000 * 10**6);
        emit TokensMinted(msg.sender, 1000000 * 10**6, block.timestamp);
    }
    
    /**
     * @dev Override decimals to return 6 (USDT standard)
     */
    function decimals() public view virtual override returns (uint8) {
        return _decimals;
    }
    
    /**
     * @dev Mint tokens (only owner)
     * @param to Recipient address
     * @param amount Amount to mint (in smallest unit - 6 decimals)
     */
    function mint(address to, uint256 amount) external onlyOwner {
        require(to != address(0), "Cannot mint to zero address");
        require(amount > 0, "Amount must be greater than 0");
        
        _mint(to, amount);
        emit TokensMinted(to, amount, block.timestamp);
    }
    
    /**
     * @dev Burn tokens from caller
     * @param amount Amount to burn (in smallest unit - 6 decimals)
     */
    function burn(uint256 amount) external {
        require(amount > 0, "Amount must be greater than 0");
        require(balanceOf(msg.sender) >= amount, "Insufficient balance");
        
        _burn(msg.sender, amount);
        emit TokensBurned(msg.sender, amount, block.timestamp);
    }
    
    /**
     * @dev Burn tokens from specific address (only owner)
     * @param from Address to burn from
     * @param amount Amount to burn (in smallest unit - 6 decimals)
     */
    function burnFrom(address from, uint256 amount) external onlyOwner {
        require(from != address(0), "Cannot burn from zero address");
        require(amount > 0, "Amount must be greater than 0");
        require(balanceOf(from) >= amount, "Insufficient balance");
        
        _burn(from, amount);
        emit TokensBurned(from, amount, block.timestamp);
    }
    
    /**
     * @dev Get token info
     * @return tokenName Token name
     * @return tokenSymbol Token symbol
     * @return tokenDecimals Token decimals
     * @return tokenTotalSupply Total supply
     */
    function getTokenInfo() external view returns (
        string memory tokenName,
        string memory tokenSymbol,
        uint8 tokenDecimals,
        uint256 tokenTotalSupply
    ) {
        return (
            name(),
            symbol(),
            decimals(),
            totalSupply()
        );
    }
}
