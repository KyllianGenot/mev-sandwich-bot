use alloy::sol;

sol! {
    struct ExactInputSingleParams {
        address tokenIn;
        address tokenOut;
        uint256 amountIn;
        uint24 fee;
        uint160 sqrtPriceLimitX96;
    }

    #[sol(rpc)]
    contract SandwichExecutor {
        function frontrun(
            address tokenIn,
            address tokenOut,
            uint24 fee,
            uint256 amountIn,
            uint256 amountOutMin
        ) external returns (uint256 amountOut);

        function backrun(
            address tokenIn,
            address tokenOut,
            uint24 fee,
            uint256 amountIn,
            uint256 amountOutMin
        ) external returns (uint256 amountOut);

        function getBalance(address token) external view returns (uint256);
        function withdrawAll(address token) external;
    }

    #[sol(rpc)]
    contract IQuoterV2 {
        function quoteExactInputSingle(ExactInputSingleParams memory params) external returns (
            uint256 amountOut,
            uint160 sqrtPriceX96After,
            uint32 initializedTicksCrossed,
            uint256 gasEstimate
        );
    }

    #[sol(rpc)]
    contract IUniswapV3Pool {
        function liquidity() external view returns (uint128);
        
        function slot0() external view returns (
            uint160 sqrtPriceX96,
            int24 tick,
            uint16 observationIndex,
            uint16 observationCardinality,
            uint16 observationCardinalityNext,
            uint8 feeProtocol,
            bool unlocked
        );
    }
}