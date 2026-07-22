use serde::Serialize;
use serde_big_array::BigArray;
use solana_short_vec as short_vec;

pub use solana_hash::Hash;
pub use solana_instruction::{AccountMeta, Instruction};
pub use solana_pubkey::Pubkey;

/// A target-independent builder for an unsigned legacy Solana transaction.
///
/// The resulting bytes include empty signature slots and can be passed directly
/// to a wallet adapter for signing and submission.
pub struct Transaction {
    payer: Pubkey,
    recent_blockhash: Hash,
    instructions: Vec<Instruction>,
}

impl Transaction {
    pub fn new(payer: Pubkey, recent_blockhash: Hash) -> Self {
        Self {
            payer,
            recent_blockhash,
            instructions: Vec::new(),
        }
    }

    pub fn instruction(mut self, instruction: Instruction) -> Self {
        self.instructions.push(instruction);
        self
    }

    pub fn instructions(mut self, instructions: impl IntoIterator<Item = Instruction>) -> Self {
        self.instructions.extend(instructions);
        self
    }

    pub fn serialize_unsigned(&self) -> Result<Vec<u8>, String> {
        let mut keys = vec![KeyMeta {
            key: self.payer,
            signer: true,
            writable: true,
            order: 0,
        }];
        for instruction in &self.instructions {
            for account in &instruction.accounts {
                Self::merge_key(
                    &mut keys,
                    account.pubkey,
                    account.is_signer,
                    account.is_writable,
                );
            }
            Self::merge_key(&mut keys, instruction.program_id, false, false);
        }
        keys.sort_by_key(|meta| {
            let bucket = match (meta.signer, meta.writable) {
                (true, true) => 0,
                (true, false) => 1,
                (false, true) => 2,
                (false, false) => 3,
            };
            (bucket, meta.order)
        });

        let required = Self::as_u8(keys.iter().filter(|key| key.signer).count(), "signers")?;
        let readonly_signed = Self::as_u8(
            keys.iter().filter(|key| key.signer && !key.writable).count(),
            "readonly signers",
        )?;
        let readonly_unsigned = Self::as_u8(
            keys.iter().filter(|key| !key.signer && !key.writable).count(),
            "readonly accounts",
        )?;
        let account_keys: Vec<_> = keys.iter().map(|meta| meta.key).collect();
        let instructions = self
            .instructions
            .iter()
            .map(|instruction| Self::compile_instruction(instruction, &account_keys))
            .collect::<Result<Vec<_>, _>>()?;

        bincode::serialize(&UnsignedTransaction {
            signatures: vec![Signature([0; 64]); required as usize],
            message: LegacyMessage {
                header: MessageHeader {
                    num_required_signatures: required,
                    num_readonly_signed_accounts: readonly_signed,
                    num_readonly_unsigned_accounts: readonly_unsigned,
                },
                account_keys,
                recent_blockhash: self.recent_blockhash,
                instructions,
            },
        })
        .map_err(|cause| cause.to_string())
    }

    fn merge_key(keys: &mut Vec<KeyMeta>, key: Pubkey, signer: bool, writable: bool) {
        if let Some(existing) = keys.iter_mut().find(|existing| existing.key == key) {
            existing.signer |= signer;
            existing.writable |= writable;
        } else {
            keys.push(KeyMeta {
                key,
                signer,
                writable,
                order: keys.len(),
            });
        }
    }

    fn compile_instruction(
        instruction: &Instruction,
        keys: &[Pubkey],
    ) -> Result<CompiledInstruction, String> {
        let index = |key: &Pubkey| {
            keys.iter()
                .position(|candidate| candidate == key)
                .ok_or_else(|| format!("Transaction account {key} is missing"))
                .and_then(|index| Self::as_u8(index, "account index"))
        };
        Ok(CompiledInstruction {
            program_id_index: index(&instruction.program_id)?,
            accounts: instruction
                .accounts
                .iter()
                .map(|account| index(&account.pubkey))
                .collect::<Result<_, _>>()?,
            data: instruction.data.clone(),
        })
    }

    fn as_u8(value: usize, label: &str) -> Result<u8, String> {
        u8::try_from(value).map_err(|_| format!("Too many {label} in transaction"))
    }
}

#[derive(Clone, Copy, Serialize)]
struct MessageHeader {
    num_required_signatures: u8,
    num_readonly_signed_accounts: u8,
    num_readonly_unsigned_accounts: u8,
}

#[derive(Serialize)]
struct CompiledInstruction {
    program_id_index: u8,
    #[serde(with = "short_vec")]
    accounts: Vec<u8>,
    #[serde(with = "short_vec")]
    data: Vec<u8>,
}

#[derive(Serialize)]
struct LegacyMessage {
    header: MessageHeader,
    #[serde(with = "short_vec")]
    account_keys: Vec<Pubkey>,
    recent_blockhash: Hash,
    #[serde(with = "short_vec")]
    instructions: Vec<CompiledInstruction>,
}

#[derive(Serialize)]
struct UnsignedTransaction {
    #[serde(with = "short_vec")]
    signatures: Vec<Signature>,
    message: LegacyMessage,
}

#[derive(Clone, Serialize)]
struct Signature(#[serde(with = "BigArray")] [u8; 64]);

#[derive(Clone, Copy)]
struct KeyMeta {
    key: Pubkey,
    signer: bool,
    writable: bool,
    order: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_an_unsigned_legacy_transaction() {
        let bytes = Transaction::new(Pubkey::new_unique(), Hash::default())
            .instruction(Instruction::new_with_bytes(
                Pubkey::new_unique(),
                &[1, 2, 3],
                vec![AccountMeta::new(Pubkey::new_unique(), false)],
            ))
            .serialize_unsigned()
            .unwrap();
        assert!(!bytes.is_empty());
        assert_eq!(bytes[0], 1);
    }
}
