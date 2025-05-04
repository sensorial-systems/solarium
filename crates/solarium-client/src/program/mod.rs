use crate::prelude::*;

use super::{Connection, Subscription};
use borsh::BorshDeserialize;
use solana_account_decoder_client_types::UiAccountEncoding;
use solana_client::rpc_config::{RpcProgramAccountsConfig, RpcAccountInfoConfig};
use solana_sdk::commitment_config::CommitmentConfig;

#[async_trait]
pub trait Program {
    type MessageBuilder;

    fn id() -> Pubkey;
    fn connection(&self) -> &Connection;
    fn message_builder(&self) -> Self::MessageBuilder;

    async fn subscribe<T>(&self) -> Result<Subscription<T>>
    where T: BorshDeserialize
    {
        self.subscribe_with_commitment(CommitmentConfig::finalized()).await
    }

    async fn subscribe_with_commitment<T>(&self, commitment: CommitmentConfig) -> Result<Subscription<T>>
    where T: BorshDeserialize
    {
        Ok(Subscription::program(self.connection(), Self::id(), Some(commitment)).await?)
    }

    async fn fetch<T>(&self) -> Result<Vec<T>>
    where T: BorshDeserialize
    {
        self.fetch_with_commitment::<T>(CommitmentConfig::finalized()).await
    }

    async fn fetch_with_commitment<T>(&self, commitment: CommitmentConfig) -> Result<Vec<T>>
    where T: BorshDeserialize
    {
        let config = RpcProgramAccountsConfig {
            account_config: RpcAccountInfoConfig {
                encoding: Some(UiAccountEncoding::Base64),
                commitment: Some(commitment),
                ..Default::default()
            },
            ..Default::default()
        };
        let accounts = self.connection().get_program_accounts_with_config(&Self::id(), config).await?;
        let images = accounts
            .iter()
            .filter_map(|(_, account)| borsh::BorshDeserialize::deserialize(&mut &account.data[..]).ok())
            .collect::<Vec<_>>();
        Ok(images)
    }
}