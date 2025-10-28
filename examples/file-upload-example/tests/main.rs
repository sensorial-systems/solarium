use solarium_client::{prelude::*, Program};
use solarium_client::utils::KeypairExt;
use solarium_client::{Account, Connection, Keypair};
use solana_sdk::compute_budget::ComputeBudgetInstruction;

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

    let size: usize = 64 * 1024;
    let mut file_bytes = vec![0u8; size];
    for i in 0..size {
        file_bytes[i] = (i % 256) as u8;
    }
    let expected_crc32 = crc32_ieee(&file_bytes);
    println!("Preparing upload: {} bytes, expected CRC32=0x{:08x}", size, expected_crc32);
    let file_size = file_bytes.len() as u64;

    // Initialize in its own transaction with empty bytes
    println!("Initializing header (payload_len=0, file_size={})...", file_size);
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

    match init_sig {
        Ok(sig) => println!("Initialized header. sig={}", sig),
        Err(e) => panic!("{:#?}", e),
    }

    // Grow the PDA in 10_240-byte steps, then upload 512B chunks per tx
    let mut grown = 0usize;
    while grown < size {
        let step = core::cmp::min(10_240, size - grown);
        println!("Growing payload: {} -> {} (+{})", grown, grown + step, step);
        let grow_sig = program
            .message_builder()
            .grow(
                keypair.pubkey(),
                file_pda.address(),
                solana_sdk::system_program::ID,
                step as u64,
            )?
            .sign(&[&keypair], Some(&keypair.pubkey())).await?
            .send_and_confirm().await;
        match grow_sig {
            Ok(sig) => println!("Grew to {} bytes. sig={}", grown + step, sig),
            Err(e) => panic!("{:#?}", e),
        }
        grown += step;
    }

    let chunk_len = 512usize;
    let mut offset = 0usize;
    while offset < size {
        let end = core::cmp::min(offset + chunk_len, size);
        let chunk = file_bytes[offset..end].to_vec();
        if offset % 10_240 == 0 || end == size {
            println!("Uploading chunk: {}..{} ({} bytes)", offset, end, end - offset);
        }
        let up_sig = program
            .message_builder()
            .upload_chunk(
                keypair.pubkey(),
                file_pda.address(),
                UploadChunkArgs { offset: offset as u64, data: chunk },
            )?
            .sign(&[&keypair], Some(&keypair.pubkey())).await?
            .send_and_confirm().await;
        if let Ok(sig) = up_sig {
            if end % 10_240 == 0 || end == size {
                println!("Uploaded through {} bytes. sig={}", end, sig);
            }
        } else if let Err(e) = up_sig { panic!("{:#?}", e) }
        offset = end;
    }

    // Final check in its own transaction with max compute budget
    let mut builder = program.message_builder();
    println!("Checking CRC on-chain with max compute budget...");
    builder.message.instructions.push(ComputeBudgetInstruction::set_compute_unit_limit(1_400_000));
    let signature = builder
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

    // Fetch raw account data and validate payload contents
    let account_data = program.connection().get_account_data(&file_pda.address()).await?;
    let header_len = borsh::to_vec(&onchain).unwrap().len();
    let payload = &account_data[header_len..header_len + (onchain.file_size as usize)];
    assert_eq!(payload, &file_bytes[..]);
    println!("Verified payload and CRC successfully.");

    Ok(())
}
