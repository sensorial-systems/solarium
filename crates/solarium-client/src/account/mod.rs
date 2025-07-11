use crate::prelude::*;
use crate::{Connection, Subscription};

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
        self.address.clone().into()
    }

    pub fn pda<S: Seeds>(connection: &Connection, seeds: S) -> Self
    where T: Owner
    {
        let address = Pubkey::find_program_address(seeds.seeds().iter().map(|s| s.as_slice()).collect::<Vec<_>>().as_slice(), T::owner()).0;
        Self::new(address, connection)
    }

    /// TODO: Unify this in a single API for Program and Client.
    pub async fn data(&self) -> Result<T>
    where T: BorshDeserialize
    {
        let data = self.connection.get_account_data(&solana_sdk::pubkey::Pubkey::from(self.address.0.clone())).await?;
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
        Ok(Subscription::account(&self.connection, self.address(), Some(commitment)).await?)
    }
}
