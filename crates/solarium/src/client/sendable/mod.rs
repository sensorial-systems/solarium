use solana_sdk::{transaction::Transaction, message::Message};

pub enum Sendable {
    Transaction(Transaction),
    Message(Message),
}

impl From<Transaction> for Sendable {
    fn from(transaction: Transaction) -> Self {
        Self::Transaction(transaction)
    }
}

impl From<Message> for Sendable {
    fn from(message: Message) -> Self {
        Self::Message(message)
    }
}

impl From<Sendable> for Transaction {
    fn from(sendable: Sendable) -> Self {
        match sendable {
            Sendable::Transaction(transaction) => transaction,
            Sendable::Message(message) => Transaction::new_unsigned(message),
        }
    }
}
