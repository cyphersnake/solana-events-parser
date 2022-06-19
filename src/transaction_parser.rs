use std::{collections::HashMap, fmt::Debug};

use serde::{Deserialize, Serialize};
pub use solana_client::rpc_client::RpcClient;
pub use solana_sdk::{
    clock::UnixTimestamp,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::Signature,
    slot_history::Slot,
};
use solana_transaction_status::EncodedConfirmedTransactionWithStatusMeta;
pub use solana_transaction_status::{
    EncodedTransactionWithStatusMeta, UiInstruction, UiTransactionEncoding,
};

pub use crate::log_parser::{self, ProgramContext, ProgramLog};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    SolanaClientResult(#[from] solana_client::client_error::ClientError),
    #[error(transparent)]
    LogParseError(#[from] crate::log_parser::Error),
    #[error("Field `meta` is empty in response of {0} tx request")]
    EmptyMetaInTransaction(Signature),
    #[error("Field `meta.log_messages` is empty in response of {0} tx request")]
    EmptyLogsInTransaction(Signature),
    #[error("Field `meta.inner_instructions` is empty")]
    EmptyInnerInstructionInTransaction(Signature),
    #[error("TODO")]
    ErrorWhileDecodeTransaction(Signature),
    #[error("TODO")]
    ErrorWhileDecodeData(bs58::decode::Error),
    #[error("TODO")]
    ParsedInnerInstructionNotSupported,
    #[error("Can't find ix ctx {0:?} in logs")]
    InstructionLogsConsistencyError(InstructionContext),
}

pub trait BindTransactionLogs {
    fn bind_transaction_logs(
        &self,
        signature: Signature,
    ) -> Result<HashMap<ProgramContext, Vec<ProgramLog>>, Error>;
}
impl BindTransactionLogs for RpcClient {
    fn bind_transaction_logs(
        &self,
        signature: Signature,
    ) -> Result<HashMap<ProgramContext, Vec<ProgramLog>>, Error> {
        Ok(log_parser::parse_events(
            self.get_transaction(&signature, UiTransactionEncoding::Base58)?
                .transaction
                .meta
                .ok_or(Error::EmptyMetaInTransaction(signature))?
                .log_messages
                .ok_or(Error::EmptyLogsInTransaction(signature))?
                .as_slice(),
        )?)
    }
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub struct InstructionContext {
    program_id: Pubkey,
    call_index: usize,
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
        let msg = self
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
                                is_signer: msg.is_maybe_writable(index),
                                is_writable: msg.is_signer(index),
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
                                    is_signer: msg.is_maybe_writable(index),
                                    is_writable: msg.is_signer(index),
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

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionParsedMeta {
    pub meta: HashMap<ProgramContext, (Instruction, Vec<ProgramLog>)>,
    pub slot: Slot,
    pub block_time: Option<UnixTimestamp>,
}

pub trait BindTransactionInstructionLogs {
    fn bind_transaction_instructions_logs(
        &self,
        signature: Signature,
    ) -> Result<TransactionParsedMeta, Error>;
}
impl BindTransactionInstructionLogs for RpcClient {
    fn bind_transaction_instructions_logs(
        &self,
        signature: Signature,
    ) -> Result<TransactionParsedMeta, Error> {
        let EncodedConfirmedTransactionWithStatusMeta {
            transaction,
            slot,
            block_time,
        } = self.get_transaction(&signature, UiTransactionEncoding::Binary)?;
        let mut instructions = transaction.bind_instructions(signature)?;

        Ok(TransactionParsedMeta {
            slot,
            block_time,
            meta: log_parser::parse_events(
                transaction
                    .meta
                    .ok_or(Error::EmptyMetaInTransaction(signature))?
                    .log_messages
                    .ok_or(Error::EmptyLogsInTransaction(signature))?
                    .as_slice(),
            )?
            .into_iter()
            .map(|(ctx, events)| {
                let ix_ctx = InstructionContext {
                    program_id: ctx.program_id,
                    call_index: ctx.call_index,
                };
                let (ix, outer_ix) = instructions
                    .remove(&ix_ctx)
                    .ok_or(Error::InstructionLogsConsistencyError(ix_ctx))?;

                // TODO Add validation of outer ix
                if (outer_ix.is_none() && ctx.invoke_level.get() == 1)
                    || (outer_ix.is_some() && ctx.invoke_level.get() != 1)
                {
                    Ok((ctx, (ix, events)))
                } else {
                    Err(Error::InstructionLogsConsistencyError(ix_ctx))
                }
            })
            .collect::<Result<_, Error>>()?,
        })
    }
}
