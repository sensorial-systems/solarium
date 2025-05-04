use crate::prelude::*;
use crate::Sendable;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{commitment_config::CommitmentConfig, signature::Signature, signers::Signers, transaction::Transaction};
use shrinkwraprs::Shrinkwrap;

#[derive(Clone, Shrinkwrap)]
pub struct Connection {
    #[shrinkwrap(main_field)]
    client: std::sync::Arc<RpcClient>,
    pub(crate) ws_address: String,
}

impl Connection {
    pub fn new(rpc_address: impl Into<String>, ws_address: impl Into<String>, commitment: CommitmentConfig) -> Self {
        let client = RpcClient::new_with_commitment(rpc_address.into(), commitment);
        let client = std::sync::Arc::new(client);
        let ws_address = ws_address.into();
        Self { client, ws_address }
    }

    pub async fn sign_and_confirm<T: Signers + ?Sized>(&self, sendable: impl Into<Sendable>, signers: &T) -> Result<Signature> {
        let sendable = sendable.into();
        let blockhash = self.get_latest_blockhash().await?;
        let mut transaction = Transaction::from(sendable);
        transaction.sign(signers, blockhash);
        Ok(self.send_and_confirm_transaction(&transaction).await?)
    }
}
