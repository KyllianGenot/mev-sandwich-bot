// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";

/// @notice SwapRouter02 interface (no deadline in struct)
interface ISwapRouter02 {
    struct ExactInputSingleParams {
        address tokenIn;
        address tokenOut;
        uint24 fee;
        address recipient;
        uint256 amountIn;
        uint256 amountOutMinimum;
        uint160 sqrtPriceLimitX96;
    }

    function exactInputSingle(ExactInputSingleParams calldata params)
        external
        payable
        returns (uint256 amountOut);
}

/// @title SandwichExecutor
/// @notice Executes sandwich attack trades on Uniswap V3
/// @dev Only owner can execute trades. Designed for testnet experimentation.
contract SandwichExecutor {
    using SafeERC20 for IERC20;

    address public owner;
    
    // Uniswap V3 SwapRouter02 on Sepolia
    ISwapRouter02 public constant SWAP_ROUTER = ISwapRouter02(0x3bFA4769FB09eefC5a80d6E87c3B9C650f7Ae48E);
    
    // WETH on Sepolia
    address public constant WETH = 0xfFf9976782d46CC05630D1f6eBAb18b2324d6B14;

    event Frontrun(address indexed tokenIn, address indexed tokenOut, uint256 amountIn, uint256 amountOut);
    event Backrun(address indexed tokenIn, address indexed tokenOut, uint256 amountIn, uint256 amountOut);
    event Withdrawn(address indexed token, uint256 amount);

    modifier onlyOwner() {
        require(msg.sender == owner, "Not owner");
        _;
    }

    constructor() {
        owner = msg.sender;
    }

    /// @notice Execute frontrun swap (buy token before victim)
    function frontrun(
        address tokenIn,
        address tokenOut,
        uint24 fee,
        uint256 amountIn,
        uint256 amountOutMin
    ) external onlyOwner returns (uint256 amountOut) {
        // Approve router to spend tokenIn
        IERC20(tokenIn).forceApprove(address(SWAP_ROUTER), amountIn);

        ISwapRouter02.ExactInputSingleParams memory params = ISwapRouter02.ExactInputSingleParams({
            tokenIn: tokenIn,
            tokenOut: tokenOut,
            fee: fee,
            recipient: address(this),
            amountIn: amountIn,
            amountOutMinimum: amountOutMin,
            sqrtPriceLimitX96: 0
        });

        amountOut = SWAP_ROUTER.exactInputSingle(params);
        
        emit Frontrun(tokenIn, tokenOut, amountIn, amountOut);
    }

    /// @notice Execute backrun swap (sell token after victim)
    function backrun(
        address tokenIn,
        address tokenOut,
        uint24 fee,
        uint256 amountIn,
        uint256 amountOutMin
    ) external onlyOwner returns (uint256 amountOut) {
        // Approve router to spend tokenIn
        IERC20(tokenIn).forceApprove(address(SWAP_ROUTER), amountIn);

        ISwapRouter02.ExactInputSingleParams memory params = ISwapRouter02.ExactInputSingleParams({
            tokenIn: tokenIn,
            tokenOut: tokenOut,
            fee: fee,
            recipient: address(this),
            amountIn: amountIn,
            amountOutMinimum: amountOutMin,
            sqrtPriceLimitX96: 0
        });

        amountOut = SWAP_ROUTER.exactInputSingle(params);
        
        emit Backrun(tokenIn, tokenOut, amountIn, amountOut);
    }

    /// @notice Get token balance of this contract
    function getBalance(address token) external view returns (uint256) {
        return IERC20(token).balanceOf(address(this));
    }

    /// @notice Withdraw tokens from contract
    function withdraw(address token, uint256 amount) external onlyOwner {
        IERC20(token).safeTransfer(owner, amount);
        emit Withdrawn(token, amount);
    }

    /// @notice Withdraw all of a token
    function withdrawAll(address token) external onlyOwner {
        uint256 balance = IERC20(token).balanceOf(address(this));
        require(balance > 0, "No balance");
        IERC20(token).safeTransfer(owner, balance);
        emit Withdrawn(token, balance);
    }

    /// @notice Withdraw ETH
    function withdrawETH() external onlyOwner {
        uint256 balance = address(this).balance;
        require(balance > 0, "No ETH balance");
        (bool success, ) = owner.call{value: balance}("");
        require(success, "ETH transfer failed");
    }

    /// @notice Transfer ownership
    function transferOwnership(address newOwner) external onlyOwner {
        require(newOwner != address(0), "Invalid owner");
        owner = newOwner;
    }

    /// @notice Receive ETH
    receive() external payable {}
}
