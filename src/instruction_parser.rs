use std::{collections::HashMap, fmt::Debug};

pub use solana_client::rpc_client::RpcClient;
pub use solana_sdk::{
    clock::UnixTimestamp,
    instruction::{AccountMeta, Instruction},
    message::VersionedMessage,
    pubkey::Pubkey,
    signature::Signature,
    slot_history::Slot,
};
pub use solana_transaction_status::{
    EncodedTransactionWithStatusMeta, UiInstruction, UiTransactionEncoding,
};

pub use crate::log_parser::{self, ProgramContext, ProgramLog};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Field `meta` is empty in response of {0} tx request")]
    EmptyMetaInTransaction(Signature),
    #[error("Field `meta.inner_instructions` is empty")]
    EmptyInnerInstructionInTransaction(Signature),
    #[error("TODO")]
    ErrorWhileDecodeTransaction(Signature),
    #[error("TODO")]
    ErrorWhileDecodeData(bs58::decode::Error),
    #[error("TODO")]
    ParsedInnerInstructionNotSupported,
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub struct InstructionContext {
    pub program_id: Pubkey,
    pub call_index: usize,
}

pub type OuterInstruction = Option<Pubkey>;

pub trait BindInstructions {
    fn bind_instructions(
        &self,
        signature: Signature,
    ) -> Result<HashMap<InstructionContext, (Instruction, OuterInstruction)>, Error>;
}
impl BindInstructions for EncodedTransactionWithStatusMeta {
    fn bind_instructions(
        &self,
        signature: Signature,
    ) -> Result<HashMap<InstructionContext, (Instruction, OuterInstruction)>, Error> {
        let msg: VersionedMessage = self
            .transaction
            .decode()
            .ok_or(Error::ErrorWhileDecodeTransaction(signature))?
            .message;

        let accounts = msg.static_account_keys();

        let mut call_index_map = HashMap::new();
        let mut get_and_update_call_index = move |program_id| {
            let i = call_index_map.entry(program_id).or_insert(0);
            let call_index = *i;
            *i += 1;
            call_index
        };

        let inner_instructions = self
            .meta
            .as_ref()
            .ok_or(Error::EmptyMetaInTransaction(signature))?
            .inner_instructions
            .as_ref()
            .ok_or(Error::EmptyInnerInstructionInTransaction(signature))?
            .iter()
            .map(|ui_ix| (ui_ix.index as usize, &ui_ix.instructions))
            .collect::<HashMap<_, _>>();

        log::trace!(
            "Inner instructions: {:?} of {}",
            inner_instructions,
            signature
        );

        let mut result = HashMap::new();
        for (ix_index, compiled_ix) in msg.instructions().iter().enumerate() {
            log::trace!("Start handling instruction with index: {}", ix_index);

            let program_id = accounts[compiled_ix.program_id_index as usize];

            let ctx = InstructionContext {
                program_id,
                call_index: get_and_update_call_index(program_id),
            };
            log::trace!("InstructionContext of {} ix is {:?}", ix_index, ctx);
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
                log::trace!(
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
                    log::trace!(
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
