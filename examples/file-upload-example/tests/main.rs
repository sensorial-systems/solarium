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

    // Prepare ~10KB file where byte i = i % 256 (limit: inner create max 10_240 bytes total)
    // Borsh header overhead for FileAccount { u32, u64, u32, Vec<u8> } is 20 bytes (4+8+4+4)
    // So max payload to fit is 10_240 - 20 = 10_220 bytes
    let size: usize = 10 * 1024 - 20;
    let mut file_bytes = vec![0u8; size];
    for i in 0..size {
        file_bytes[i] = (i % 256) as u8;
    }
    let expected_crc32 = crc32_ieee(&file_bytes);
    let file_size = file_bytes.len() as u64;

    // Initialize in its own transaction
    let init_sig = program
        .message_builder()
        .initialize(
            keypair.pubkey(),
            file_pda.address(),
            solana_sdk::system_program::ID,
            InitArgs { expected_crc32, file_size },
        )?
        .sign(&[&keypair], Some(&keypair.pubkey())).await?
        .send_and_confirm().await;

    if let Err(e) = init_sig { panic!("{:#?}", e) }

    // Upload in 1KB chunks, one transaction per chunk to avoid tx size limits
    let chunk_len = 512usize;
    let mut offset = 0usize;
    while offset < size {
        let end = core::cmp::min(offset + chunk_len, size);
        let chunk = file_bytes[offset..end].to_vec();
        let up_sig = program
            .message_builder()
            .upload_chunk(
                keypair.pubkey(),
                file_pda.address(),
                UploadChunkArgs { offset: offset as u64, data: chunk },
            )?
            .sign(&[&keypair], Some(&keypair.pubkey())).await?
            .send_and_confirm().await;
        if let Err(e) = up_sig { panic!("{:#?}", e) }
        offset = end;
    }

    // Final check in its own transaction
    let signature = program
        .message_builder()
        .check_crc(keypair.pubkey(), file_pda.address())?
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


