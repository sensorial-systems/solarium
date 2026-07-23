use crate::prelude::*;
use crate::{Connection, Message};

use solana_sdk::hash::Hash;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signature;
use solana_sdk::signers::Signers;

pub struct Transaction {
    connection: Connection,
    transaction: solana_sdk::transaction::Transaction,
}

impl Transaction {
    pub fn new<T: Signers + ?Sized>(
        connection: &Connection,
        message: &Message,
        payer: Option<&Pubkey>,
        signers: &T,
        blockhash: Hash,
    ) -> Self {
        let connection = connection.clone();
        let transaction = solana_sdk::transaction::Transaction::new_signed_with_payer(
            &message.instructions,
            payer,
            signers,
            blockhash,
        );
        Self {
            connection,
            transaction,
        }
    }

    pub async fn send(self) -> Result<Signature> {
        let signature = self.connection.send_transaction(&self.transaction).await?;
        Ok(signature)
    }

    pub async fn send_and_confirm(self) -> Result<Signature> {
        let signature = self
            .connection
            .send_and_confirm_transaction(&self.transaction)
            .await?;
        Ok(signature)
    }
}
