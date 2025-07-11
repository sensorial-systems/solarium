use std::pin::Pin;

use crate::prelude::*;
use crate::Connection;

use borsh::BorshDeserialize;
use futures::future::BoxFuture;
use futures::{Stream, StreamExt};
use shrinkwraprs::Shrinkwrap;
use solana_account_decoder_client_types::UiAccountEncoding;
use solana_client::nonblocking::pubsub_client::PubsubClient;
use solana_client::rpc_config::{RpcAccountInfoConfig, RpcProgramAccountsConfig};
use solana_sdk::commitment_config::CommitmentConfig;

#[derive(Shrinkwrap)]
#[shrinkwrap(mutable)]
pub struct Subscription<T: BorshDeserialize> {
    #[shrinkwrap(main_field)]
    pub receiver: Pin<Box<dyn Stream<Item = T>>>,
    unsubscribe_function: Box<dyn FnOnce() -> BoxFuture<'static, ()> + Send>,
}

impl<T: BorshDeserialize> Subscription<T> {
    pub async fn account(connection: &Connection, address: Pubkey, commitment: Option<CommitmentConfig>) -> Result<Self> {
        let client = PubsubClient::new(&connection.ws_address).await?;
        let config = RpcAccountInfoConfig {
            encoding: Some(UiAccountEncoding::JsonParsed),
            commitment,
            ..Default::default()
        };
        // FIXME: This is a hack to make the client live for the lifetime of the subscription
        let client = Box::leak(Box::new(client));
        let (receiver, unsubscribe_function) = client.account_subscribe(&solana_sdk::pubkey::Pubkey::from(address.0.clone()), Some(config)).await?;
        let receiver = receiver
            .filter_map(|account| async move {
                account.value.data.decode()
            })
            .filter_map(|data| async move {
                BorshDeserialize::deserialize(&mut &data[..]).ok()
            });
        let receiver = Box::pin(receiver);

        Ok(Self { receiver, unsubscribe_function })
    }

    pub async fn program(connection: &Connection, program_id: Pubkey, commitment: Option<CommitmentConfig>) -> Result<Self> {
        let client = PubsubClient::new(&connection.ws_address).await?;
        let config = RpcProgramAccountsConfig {
            filters: None,
            account_config: RpcAccountInfoConfig {
                encoding: Some(UiAccountEncoding::Base64),
                commitment,
                ..Default::default()
            },
            ..Default::default()
        };
        // FIXME: This is a hack to make the client live for the lifetime of the subscription
        let client = Box::leak(Box::new(client));
        let (receiver, unsubscribe_function) = client.program_subscribe(&solana_sdk::pubkey::Pubkey::from(program_id.0.clone()), Some(config)).await?;
        let receiver = receiver
            .filter_map(|account| async move {
                account.value.account.data.decode()
            })
            .filter_map(|data| async move {
                BorshDeserialize::deserialize(&mut &data[..]).ok()
            });
        let receiver = Box::pin(receiver);

        Ok(Self { receiver, unsubscribe_function })
    }

    pub async fn unsubscribe(self) {
        (self.unsubscribe_function)().await
    }
}
