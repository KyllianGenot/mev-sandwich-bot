use crate::contracts::SandwichExecutor;
use crate::analysis::simulator::Simulator;
use alloy::primitives::{Address, U256};
use alloy::providers::Provider;
use eyre::Result;
use tracing::{info, warn, debug};
use futures::future::join_all;

pub struct BundleExecutor<P> {
    provider: P,
    executor_address: Address,
    simulator: Simulator<P>,
}

impl<P: Provider + Clone + 'static> BundleExecutor<P> {
    pub fn new(provider: P, executor_address: Address) -> Self {
        Self {
            provider: provider.clone(),
            executor_address,
            simulator: Simulator::new(provider),
        }
    }

    pub async fn execute_optimized(
        &self,
        token_in: Address,
        token_out: Address,
        frontrun_amount: U256,
        victim_gas_price: u128,
    ) -> Result<(U256, U256)> {
        let executor = SandwichExecutor::new(self.executor_address, &self.provider);
        let fee_tiers = vec![3000, 500, 10000];

        let check_futures = fee_tiers.iter().map(|&fee| {
            let sim_ref = &self.simulator;
            async move {
                match sim_ref.check_profitability(token_in, token_out, fee, frontrun_amount).await {
                    Ok(true) => match sim_ref.simulate_swap(token_in, token_out, fee, frontrun_amount).await {
                        Ok(sim) if sim.profitable => Some((fee, sim.amount_out)),
                        _ => None,
                    },
                    _ => None
                }
            }
        });

        let results = join_all(check_futures).await;
        let (fee, _est) = match results.into_iter().find_map(|r| r) {
            Some(res) => {
                info!("   ✨ Sim Passed! Fee: {}", res.0);
                res
            },
            None => {
                debug!("   ⏭️  No profitable pool found");
                return Ok((U256::ZERO, U256::ZERO));
            }
        };

        let gas_price = if victim_gas_price > 0 {
            victim_gas_price + (victim_gas_price / 5)
        } else {
            2_000_000_000
        };

        info!("   🚀 Executing Frontrun...");

        let frontrun_tx = executor
            .frontrun(token_in, token_out, fee.try_into().unwrap(), frontrun_amount, U256::ZERO)
            .gas_price(gas_price)
            .gas(400_000u64)
            .send()
            .await?;

        let frontrun_receipt = frontrun_tx.get_receipt().await?;
        if !frontrun_receipt.status() {
            warn!("   ❌ Frontrun reverted");
            return Ok((U256::ZERO, U256::ZERO));
        }

        let tokens_held = executor.getBalance(token_out).call().await?; 
        if tokens_held.is_zero() {
            warn!("   ⚠️ Zero tokens");
            return Ok((U256::ZERO, U256::ZERO));
        }

        info!("   📦 Received {}, Backrunning...", tokens_held);
        
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        let weth_before = executor.getBalance(token_in).call().await?;
        
        let backrun_tx = executor
            .backrun(token_out, token_in, fee.try_into().unwrap(), tokens_held, U256::ZERO)
            .gas(400_000u64)
            .send()
            .await?;
        
        let backrun_receipt = backrun_tx.get_receipt().await?;
        
        if backrun_receipt.status() {
            let weth_after = executor.getBalance(token_in).call().await?;
            let revenue = if weth_after > weth_before { weth_after - weth_before } else { U256::ZERO };
            
            let cost = (U256::from(frontrun_receipt.gas_used) * U256::from(frontrun_receipt.effective_gas_price)) +
                       (U256::from(backrun_receipt.gas_used) * U256::from(backrun_receipt.effective_gas_price));

            if revenue > cost {
                info!("   ✅ PROFIT! Net: {} WEI", revenue - cost);
                Ok((tokens_held, revenue - cost))
            } else {
                warn!("   🔻 LOSS: Rev {} < Gas {}", revenue, cost);
                Ok((tokens_held, U256::ZERO))
            }
        } else {
            warn!("   ❌ Backrun reverted");
            Ok((tokens_held, U256::ZERO))
        }
    }
}