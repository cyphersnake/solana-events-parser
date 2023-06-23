use std::{collections::HashMap, fmt::Debug, str::FromStr};

pub use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::ParsePubkeyError;
pub use solana_sdk::{
    clock::UnixTimestamp,
    instruction::{AccountMeta, Instruction},
    message::VersionedMessage,
    pubkey::Pubkey,
    signature::Signature,
    slot_history::Slot,
};
pub use solana_transaction_status::{
    option_serializer::OptionSerializer, EncodedTransactionWithStatusMeta, UiInstruction,
    UiTransactionEncoding,
};
use solana_transaction_status::{UiLoadedAddresses, UiTransactionStatusMeta};

pub use crate::log_parser::{self, ProgramContext, ProgramLog};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Field `meta` is empty in response of {0} tx request")]
    EmptyMetaInTransaction(Signature),
    #[error("Field `meta.inner_instructions` is empty")]
    EmptyInnerInstructionInTransaction(Signature),
    #[error("Error while decode transaction {0}")]
    ErrorWhileDecodeTransaction(Signature),
    #[error("Error while decode data {0:?}")]
    ErrorWhileDecodeData(bs58::decode::Error),
    #[error("Parsed inner instruction not supported")]
    ParsedInnerInstructionNotSupported,
    #[error("Pubkey parse error {0:?}")]
    PubkeyParseError(#[from] ParsePubkeyError),
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub struct InstructionContext {
    pub program_id: Pubkey,
    pub call_index: usize,
}

pub type OuterInstructionProgramId = Option<Pubkey>;

pub trait GetLoadedAccounts {
    fn get_loaded_accounts(&self) -> Option<Result<Vec<Pubkey>, Error>>;
}
impl GetLoadedAccounts for EncodedTransactionWithStatusMeta {
    fn get_loaded_accounts(&self) -> Option<Result<Vec<Pubkey>, Error>> {
        let msg = self.transaction.decode()?.message;

        let additional_accounts = match &self.meta {
            Some(UiTransactionStatusMeta {
                loaded_addresses: OptionSerializer::Some(UiLoadedAddresses { writable, readonly }),
                ..
            }) => writable
                .iter()
                .map(|key| Pubkey::from_str(key))
                .chain(readonly.iter().map(|key| Pubkey::from_str(key)))
                .collect(),
            _ => vec![],
        };

        Some(
            msg.static_account_keys()
                .iter()
                .copied()
                .map(Ok)
                .chain(additional_accounts.into_iter())
                .collect::<Result<Vec<Pubkey>, ParsePubkeyError>>()
                .map_err(Error::from),
        )
    }
}

/// [`BindInstructions`] trait provides a method to bind an `Instruction` to its context.
pub trait BindInstructions {
    /// Bind instructions the transaction into separate contexts.
    ///
    /// As decoding errors are possible and the original instruction signature
    /// may not be obtained, it is passed in parameters.
    fn bind_instructions(
        &self,
        signature: Signature,
    ) -> Result<HashMap<InstructionContext, (Instruction, OuterInstructionProgramId)>, Error>;
}
impl BindInstructions for EncodedTransactionWithStatusMeta {
    /// Bind instructions the transaction into separate contexts.
    ///
    /// As decoding errors are possible and the original instruction signature
    /// may not be obtained, it is passed in parameters.
    ///
    /// It starts by decoding the transaction and loading the accounts. Then it iterates through
    /// all instructions of the transaction and binds each instruction to its context. The context
    /// is created by using the program id of the instruction and a call index which is incrementing
    /// with each call of the same program id.
    fn bind_instructions(
        &self,
        signature: Signature,
    ) -> Result<HashMap<InstructionContext, (Instruction, OuterInstructionProgramId)>, Error> {
        let tx = self.transaction.decode().ok_or_else(|| {
            tracing::error!("Can't decode transaction");
            Error::ErrorWhileDecodeTransaction(signature)
        })?;

        if tx.signatures.first().ne(&Some(&signature)) {
            use itertools::Itertools;
            tracing::error!(
                "Signature not match {}, {}",
                signature,
                tx.signatures.iter().map(ToString::to_string).join(", ")
            );
            return Err(Error::ErrorWhileDecodeTransaction(signature));
        }

        let msg = tx.message;

        let accounts = self
            .get_loaded_accounts()
            .ok_or(Error::ErrorWhileDecodeTransaction(signature))??;

        let mut call_index_map = HashMap::new();
        let mut get_and_update_call_index = move |program_id| {
            let i = call_index_map.entry(program_id).or_insert(0);
            let call_index = *i;
            *i += 1;
            call_index
        };

        let inner_instructions = match self
            .meta
            .as_ref()
            .ok_or(Error::EmptyMetaInTransaction(signature))?
            .inner_instructions
            .as_ref()
        {
            OptionSerializer::None | OptionSerializer::Skip => {
                Err(Error::EmptyInnerInstructionInTransaction(signature))
            }
            OptionSerializer::Some(inner_instructions) => Ok(inner_instructions
                .iter()
                .map(|ui_ix| (ui_ix.index as usize, &ui_ix.instructions))
                .collect::<HashMap<_, _>>()),
        }?;

        tracing::trace!(
            "Inner instructions: {:?} of {}",
            inner_instructions,
            signature
        );

        let mut result = HashMap::new();
        for (ix_index, compiled_ix) in msg.instructions().iter().enumerate() {
            tracing::trace!("Start handling instruction with index: {}", ix_index);

            let program_id = accounts[compiled_ix.program_id_index as usize];

            let ctx = InstructionContext {
                program_id,
                call_index: get_and_update_call_index(program_id),
            };
            tracing::trace!("InstructionContext of {} ix is {:?}", ix_index, ctx);
            result.insert(
                ctx,
                (
                    Instruction {
                        program_id,
                        accounts: compiled_ix
                            .accounts
                            .iter()
                            .map(|&index| index as usize)
                            .map(|index| AccountMeta {
                                pubkey: accounts[index],
                                is_signer: msg.is_signer(index),
                                is_writable: msg.is_maybe_writable(index),
                            })
                            .collect(),
                        data: compiled_ix.data.clone(),
                    },
                    None,
                ),
            );
            if let Some(invokes) = inner_instructions.get(&ix_index) {
                tracing::trace!(
                    "Found inner instruction {} for {} transaction instruction",
                    invokes.len(),
                    ix_index
                );
                for (invoke_index, invoke) in invokes.iter().enumerate() {
                    let invoke_ix = match invoke {
                        UiInstruction::Compiled(compiled) => Instruction {
                            program_id: accounts[compiled.program_id_index as usize],
                            accounts: compiled
                                .accounts
                                .iter()
                                .map(|&index| index as usize)
                                .map(|index| AccountMeta {
                                    pubkey: accounts[index],
                                    is_signer: msg.is_signer(index),
                                    is_writable: msg.is_maybe_writable(index),
                                })
                                .collect(),
                            data: bs58::decode(&compiled.data)
                                .into_vec()
                                .map_err(Error::ErrorWhileDecodeData)?,
                        },
                        UiInstruction::Parsed(_parsed) => {
                            return Err(Error::ParsedInnerInstructionNotSupported);
                        }
                    };
                    let ctx = InstructionContext {
                        program_id: invoke_ix.program_id,
                        call_index: get_and_update_call_index(invoke_ix.program_id),
                    };
                    tracing::trace!(
                        "Invoke {} of ix {} with ctx {:?}",
                        invoke_index,
                        ix_index,
                        ctx
                    );
                    result.insert(ctx, (invoke_ix, Some(program_id)));
                }
            }
        }

        Ok(result)
    }
}

#[cfg(feature = "anchor")]
mod anchor {
    use std::io;

    use anchor_lang::{AnchorDeserialize, Discriminator, Owner};

    use super::Instruction;

    pub trait ParseInstruction {
        fn parse_instruction<T: Discriminator + Owner + AnchorDeserialize>(
            &self,
        ) -> Option<Result<T, io::Error>>;
    }

    impl ParseInstruction for Instruction {
        fn parse_instruction<I: Discriminator + Owner + AnchorDeserialize>(
            &self,
        ) -> Option<Result<I, io::Error>> {
            const DISCRIMINATOR_SIZE: usize = 8;
            let (discriminantor, event) = self.data.split_at(DISCRIMINATOR_SIZE);
            (I::owner().eq(&self.program_id) && I::discriminator().eq(discriminantor))
                .then(|| I::try_from_slice(event))
        }
    }
}
#[cfg(feature = "anchor")]
pub use anchor::*;
