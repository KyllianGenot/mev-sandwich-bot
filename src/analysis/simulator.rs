use crate::contracts::{ExactInputSingleParams, IQuoterV2};
use alloy::{
    primitives::{Address, U160, U256},
    providers::Provider,
};
use eyre::Result;

const QUOTER_V2: &str = "0xEd1f6473345F45b75F8179591dd5bA1888cf2FB3";

pub struct Simulator<P> {
    provider: P,
    quoter_address: Address,
}

#[derive(Debug)]
pub struct SimulationResult {
    pub amount_out: U256,
    pub profitable: bool,
}

impl<P: Provider + Clone> Simulator<P> {
    pub fn new(provider: P) -> Self {
        Self {
            provider,
            quoter_address: QUOTER_V2.parse().unwrap(),
        }
    }

    pub async fn simulate_swap(
        &self,
        token_in: Address,
        token_out: Address,
        fee: u32,
        amount_in: U256,
    ) -> Result<SimulationResult> {
        let quoter = IQuoterV2::new(self.quoter_address, &self.provider);

        let params = ExactInputSingleParams {
            tokenIn: token_in,
            tokenOut: token_out,
            amountIn: amount_in,
            fee: fee.try_into().unwrap(),
            sqrtPriceLimitX96: U160::ZERO,
        };

        match quoter.quoteExactInputSingle(params).call().await {
            Ok(quote) => Ok(SimulationResult {
                amount_out: quote.amountOut,
                profitable: true,
            }),
            Err(e) => {
                tracing::warn!(" ⚠️ Quoter Failed (Tier {}): {}", fee, e);
                Ok(SimulationResult {
                    amount_out: U256::ZERO,
                    profitable: false,
                })
            }
        }
    }

    pub async fn check_profitability(
        &self,
        token_in: Address,
        token_out: Address,
        fee: u32,
        frontrun_amount: U256,
    ) -> Result<bool> {
        let buy_sim = self
            .simulate_swap(token_in, token_out, fee, frontrun_amount)
            .await?;

        if !buy_sim.profitable || buy_sim.amount_out.is_zero() {
            return Ok(false);
        }

        let sell_sim = self
            .simulate_swap(token_out, token_in, fee, buy_sim.amount_out)
            .await?;

        if !sell_sim.profitable {
            return Ok(false);
        }

        let min_recovery = frontrun_amount;

        if sell_sim.amount_out > min_recovery {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}