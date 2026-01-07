use alloy::providers::{Provider, ProviderBuilder, WsConnect};
use alloy::rpc::types::Transaction;
use eyre::Result;
use futures::StreamExt;
use tokio::sync::mpsc;
use std::sync::Arc;

pub struct MempoolListener {
    ws_url: String,
}

impl MempoolListener {
    pub fn new(ws_url: String) -> Self {
        Self { ws_url }
    }

    pub async fn start(&self, tx_sender: mpsc::Sender<Transaction>) -> Result<()> {
        tracing::info!("Connecting to mempool stream...");
        let ws = WsConnect::new(&self.ws_url);
        let provider = ProviderBuilder::new().connect_ws(ws).await?;
        let provider = Arc::new(provider);

        tracing::info!("Connected, subscribing...");
        let sub = provider.subscribe_pending_transactions().await?;
        let mut stream = sub.into_stream();

        tracing::info!("Listening for pending transactions...");
        while let Some(tx_hash) = stream.next().await {
            let provider_clone = provider.clone();
            let sender_clone = tx_sender.clone();
            tokio::spawn(async move {
                if let Ok(Some(tx)) = provider_clone.get_transaction_by_hash(tx_hash).await {
                    let _ = sender_clone.send(tx).await;
                }
            });
        }
        Ok(())
    }
}