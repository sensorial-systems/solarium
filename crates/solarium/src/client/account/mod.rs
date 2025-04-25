use crate::prelude::*;
use borsh::BorshDeserialize;
use solana_sdk::commitment_config::CommitmentConfig;
use crate::client::{Connection, Subscription};

pub struct Account<T> {
    address: Pubkey,
    connection: Connection,
    phantom: std::marker::PhantomData<T>,
}

impl<T> Account<T> {
    pub fn new(address: Pubkey, connection: &Connection) -> Self {
        let phantom = Default::default();
        let connection = connection.clone();
        Self { address, connection, phantom }
    }

    pub fn address(&self) -> Pubkey {
        self.address
    }

    pub fn pda(connection: &Connection) -> Self
    where T: Pda + Owner
    {
        let address = Pubkey::find_program_address(T::seeds(), T::owner()).0;
        Self::new(address, connection)
    }

    /// TODO: Unify this in a single API for Program and Client.
    pub async fn data(&self) -> Result<T>
    where T: BorshDeserialize
    {
        let data = self.connection.get_account_data(&self.address).await?;
        Ok(BorshDeserialize::deserialize(&mut &data[..])?)
    }

    pub async fn subscribe(&self) -> Result<Subscription<T>>
    where T: BorshDeserialize
    {
        self.subscribe_with_commitment(CommitmentConfig::finalized()).await
    }

    pub async fn subscribe_with_commitment(&self, commitment: CommitmentConfig) -> Result<Subscription<T>>
    where T: BorshDeserialize
    {
        Ok(Subscription::account(&self.connection, self.address, Some(commitment)).await?)
    }
}
