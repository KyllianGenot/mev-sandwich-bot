mod analysis;
mod config;
mod contracts;
mod execution;
mod mempool;

use alloy::consensus::Transaction as TransactionTrait;
use alloy::network::EthereumWallet;
use alloy::primitives::{Address, U256};
use alloy::providers::ProviderBuilder;
use alloy::signers::local::PrivateKeySigner;
use alloy::sol;
use eyre::Result;
use tokio::sync::mpsc;
use tracing_subscriber::EnvFilter;

use crate::execution::bundle::BundleExecutor;
use crate::mempool::{MempoolListener, TransactionDecoder};

const WETH: &str = "0xfFf9976782d46CC05630D1f6eBAb18b2324d6B14";

sol! {
    #[sol(rpc)]
    contract SandwichExecutorHelper {
        function getBalance(address token) external view returns (uint256);
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env()
                .add_directive("mev_sandwich_bot=info".parse()?)
                .add_directive("alloy=warn".parse()?),
        )
        .init();

    tracing::info!("🥪 Starting MEV Sandwich Bot (Production Mode)");

    let config = config::Config::load()?;
    let executor_address: Address = std::env::var("EXECUTOR_ADDRESS")?.parse()?;
    
    let signer: PrivateKeySigner = config.private_key.parse()?;
    let wallet = EthereumWallet::from(signer.clone());
    
    let provider = ProviderBuilder::new()
        .wallet(wallet.clone())
        .connect_http(config.rpc_url.parse()?);

    let helper = SandwichExecutorHelper::new(executor_address, &provider);
    let weth_addr: Address = WETH.parse()?;
    
    let balance = helper.getBalance(weth_addr).call().await?; 
    tracing::info!("Executor WETH Balance: {:.4}", balance.to::<u128>() as f64 / 1e18);

    if balance.is_zero() {
        tracing::error!("⚠️ Executor has 0 WETH. You cannot frontrun without capital.");
    }

    let bundle_executor = BundleExecutor::new(provider.clone(), executor_address);

    let (tx_sender, mut tx_receiver) = mpsc::channel(5000); 

    let ws_url = config.rpc_ws_url.clone();
    let listener_handle = tokio::spawn(async move {
        let listener = MempoolListener::new(ws_url);
        if let Err(e) = listener.start(tx_sender).await {
            tracing::error!("Mempool listener crashed: {}", e);
        }
    });

    tracing::info!("📡 Scanning mempool for opportunities...");
    
    let min_victim_amount = U256::from(1_000_000_000_000_000u128); 

    while let Some(tx) = tx_receiver.recv().await {
        if !TransactionDecoder::is_dex_swap(&tx) { continue; }
        
        let Some(swap) = TransactionDecoder::decode_swap(&tx) else { continue; };
        
        let is_weth_to_token = swap.token_in == weth_addr;
        
        if !is_weth_to_token || swap.amount_in < min_victim_amount { continue; }

        let amount_eth = swap.amount_in.to::<u128>() as f64 / 1e18;
        tracing::info!("🎯 Opp: {} | {:.4} ETH -> {:?}", swap.method_name, amount_eth, swap.token_out);

        let frontrun_amount = swap.amount_in * U256::from(30) / U256::from(100);
        let victim_gas_price = tx.inner.gas_price().unwrap_or_else(|| tx.inner.max_fee_per_gas());

        let _ = bundle_executor.execute_optimized(
            swap.token_in,
            swap.token_out,
            frontrun_amount,
            victim_gas_price
        ).await;
    }

    listener_handle.await?;
    Ok(())
}