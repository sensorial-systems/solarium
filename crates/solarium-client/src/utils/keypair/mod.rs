use std::future::Future;

use crate::prelude::*;
use crate::Connection;
use solana_sdk::{signature::Keypair, signer::Signer};

pub trait KeypairExt {
    fn new_with_funds(connection: &Connection, funds: u64) -> impl Future<Output = Result<Self>>
    where
        Self: Sized;
}

impl KeypairExt for Keypair {
    async fn new_with_funds(connection: &Connection, funds: u64) -> Result<Self> {
        let keypair = Keypair::new();
        let signature = connection
            .request_airdrop(&keypair.pubkey(), funds)
            .await
            .unwrap();
        while !connection.confirm_transaction(&signature).await.unwrap() {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
        Ok(keypair)
    }
}
