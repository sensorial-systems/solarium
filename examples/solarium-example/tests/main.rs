use solarium_client::prelude::*;
use solarium_client::Program;
use solarium_client::{Account, Connection, Keypair};
use solarium_client::utils::*;

use solarium_example::{ExampleData, ExampleSeeds};

mod solarium_example_client {
    solarium_client::generate_client!("solarium-example");
}

use solarium_example_client::Example;

#[tokio::test]
async fn solana() -> Result<()> {
    let connection = Connection::new("http://localhost:8899", "ws://localhost:8900", CommitmentConfig::confirmed());
    let keypair = Keypair::new_with_funds(&connection, LAMPORTS_PER_SOL).await?;

    let data = Account::<ExampleData>::pda(&connection, ExampleSeeds::default());
    let program = Example::new(&connection);

    let signature = program
        .message_builder()
        .initialize(keypair.pubkey(), data.address(), solana_sdk::system_program::ID)?
        .set_message(keypair.pubkey(), data.address(), "hello from example".to_string())?
        .sign(&[&keypair], Some(&keypair.pubkey())).await?
        .send_and_confirm().await;

    match signature {
        Ok(signature) => println!("Executed example with signature: {}", signature),
        Err(e) => panic!("{:#?}", e),
    }

    let item = data.data().await?;
    assert!(item.message == "hello from example");

    Ok(())
}


