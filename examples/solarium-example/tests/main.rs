use solarium_client::prelude::*;
use solarium_client::utils::*;
use solarium_client::Program;
use solarium_client::{Account, Connection, Keypair};

use solarium_example::{ExampleData, ExampleSeeds};

mod solarium_example_client {
    solarium_client::generate_client!("solarium-example");
}

mod custom_address_client {
    solarium_client::generate_client!("solarium-example", "11111111111111111111111111111111");
}

use solarium_example_client::Example;

#[test]
fn uses_custom_program_address() {
    assert_eq!(
        custom_address_client::Example::id(),
        solarium_client::prelude::Pubkey::default()
    );
}

#[test]
fn dynamic_address_overrides_custom_program_address() -> Result<()> {
    let connection = Connection::new(
        "http://localhost:8899",
        "ws://localhost:8900",
        CommitmentConfig::confirmed(),
    );
    let dynamic_address = Pubkey::new_unique();
    let program = custom_address_client::Example::new(&connection).with_address(dynamic_address);

    assert_eq!(program.address(), dynamic_address);

    let builder = program.message_builder().initialize(
        Pubkey::new_unique(),
        Pubkey::new_unique(),
        solana_sdk::system_program::ID,
    )?;
    assert_eq!(builder.instructions()[0].program_id, dynamic_address);
    Ok(())
}

#[tokio::test]
async fn solana() -> Result<()> {
    let connection = Connection::new(
        "http://localhost:8899",
        "ws://localhost:8900",
        CommitmentConfig::confirmed(),
    );
    let keypair = Keypair::new_with_funds(&connection, LAMPORTS_PER_SOL).await?;

    let data = Account::<ExampleData>::pda(&connection, ExampleSeeds::default());
    let program = Example::new(&connection);

    let signature = program
        .message_builder()
        .initialize(
            keypair.pubkey(),
            data.address(),
            solana_sdk::system_program::ID,
        )?
        .set_message(
            keypair.pubkey(),
            data.address(),
            "hello from example".to_string(),
        )?
        .sign(&[&keypair], Some(&keypair.pubkey()))
        .await?
        .send_and_confirm()
        .await;

    match signature {
        Ok(signature) => println!("Executed example with signature: {}", signature),
        Err(e) => panic!("{:#?}", e),
    }

    let item = data.data().await?;
    assert!(item.message == "hello from example");

    Ok(())
}
