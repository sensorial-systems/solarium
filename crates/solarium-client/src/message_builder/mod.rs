use crate::prelude::*;

use super::{Connection, Message, Transaction};

use solana_sdk::{instruction::Instruction, signers::Signers};


pub struct MessageBuilder {
    pub connection: Connection,
    pub message: Message
}

impl MessageBuilder {
    pub fn new(connection: &Connection) -> Self {
        let connection = connection.clone();
        let message = Default::default();
        Self { connection, message }
    }

    pub fn instructions(&self) -> &[Instruction] {
        self.message.instructions()
    }

    pub async fn sign<T: Signers + ?Sized>(&self, signers: &T, payer: Option<&Pubkey>) -> Result<Transaction> {
        let payer = payer.map(|p| solana_sdk::pubkey::Pubkey::from(p.0.clone()));
        Ok(Transaction::new(&self.connection, &self.message, payer.as_ref(), signers, self.connection.get_latest_blockhash().await?))
    }
}