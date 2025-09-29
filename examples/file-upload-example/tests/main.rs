use solarium_client::{prelude::*, Program};
use solarium_client::utils::KeypairExt;
use solarium_client::{Account, Connection, Keypair};

use file_upload_example::{FileAccount, FileSeeds, InitArgs, UploadChunkArgs};

mod file_upload_example_client {
    solarium_client::generate_client!("file-upload-example");
}

use file_upload_example_client::FileUploadExample;

fn crc32_ieee(data: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFF_FFFF;
    for &byte in data {
        let mut c = (crc ^ (byte as u32)) & 0xFF;
        for _ in 0..8 {
            c = if (c & 1) != 0 { 0xEDB88320 ^ (c >> 1) } else { c >> 1 };
        }
        crc = (crc >> 8) ^ c;
    }
    !crc
}

#[tokio::test]
async fn solana() -> Result<()> {
    let connection = Connection::new("http://localhost:8899", "ws://localhost:8900", CommitmentConfig::confirmed());
    let keypair = Keypair::new_with_funds(&connection, LAMPORTS_PER_SOL).await?;

    let file_pda = Account::<FileAccount>::pda(&connection, FileSeeds::default());
    let program = FileUploadExample::new(&connection);

    // Prepare sample file bytes and expected CRC
    let file_bytes = b"The quick brown fox jumps over the lazy dog".to_vec();
    let expected_crc32 = crc32_ieee(&file_bytes);
    let file_size = file_bytes.len() as u64;

    // Split into two chunks to exercise chunked upload
    let split = 17usize;
    let chunk_a = file_bytes[..split].to_vec();
    let chunk_b = file_bytes[split..].to_vec();

    // Initialize PDA with expected CRC and total size, then upload chunks and verify
    let signature = program
        .message_builder()
        .initialize(
            keypair.pubkey(),
            file_pda.address(),
            solana_sdk::system_program::ID,
            InitArgs { expected_crc32, file_size },
        )?
        .upload_chunk(
            keypair.pubkey(),
            file_pda.address(),
            UploadChunkArgs { offset: 0, data: chunk_a.clone() },
        )?
        .upload_chunk(
            keypair.pubkey(),
            file_pda.address(),
            UploadChunkArgs { offset: split as u64, data: chunk_b.clone() },
        )?
        .check_crc(
            keypair.pubkey(),
            file_pda.address(),
        )?
        .sign(&[&keypair], Some(&keypair.pubkey())).await?
        .send_and_confirm().await;

    match signature {
        Ok(signature) => println!("Executed file upload with signature: {}", signature),
        Err(e) => panic!("{:#?}", e),
    }

    // Verify on-chain data
    let onchain = file_pda.data().await?;
    assert_eq!(onchain.file_size as usize, file_bytes.len());
    assert_eq!(onchain.expected_crc32, expected_crc32);
    assert_eq!(onchain.written_crc32, expected_crc32);
    assert_eq!(onchain.bytes, file_bytes);

    Ok(())
}


