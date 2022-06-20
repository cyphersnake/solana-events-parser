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
pub use solana_transaction_status::{
    EncodedConfirmedTransaction, EncodedTransactionWithStatusMeta, UiInstruction,
    UiTransactionEncoding,
};

pub use crate::{
    instruction_parser::{BindInstructions, InstructionContext},
    log_parser::{self, ProgramContext, ProgramLog},
};

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
    #[error(transparent)]
    InstructionParsingError(#[from] crate::instruction_parser::Error),
    #[error("Can't find ix ctx {0:?} in logs")]
    InstructionLogsConsistencyError(InstructionContext),
    #[error("Provided log and provided ix not match by owner")]
    InstructionLogsOwnerError { ix_owner: Pubkey, log_owner: Pubkey },
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

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionParsedMeta {
    pub meta: HashMap<ProgramContext, (Instruction, Vec<ProgramLog>)>,
    pub slot: Slot,
    pub block_time: Option<UnixTimestamp>,
}

#[cfg(feature = "anchor")]
mod anchor {
    use std::io;

    use anchor_lang::{AnchorDeserialize, Discriminator, Owner};

    use super::{ProgramLog, TransactionParsedMeta};

    impl TransactionParsedMeta {
        pub fn find_ix<I: Discriminator + Owner + AnchorDeserialize>(
            &self,
        ) -> Result<Vec<(I, &Vec<ProgramLog>)>, io::Error> {
            use crate::ParseInstruction;
            self.meta
                .iter()
                .filter_map(|(ctx, meta)| ctx.program_id.eq(&I::owner()).then(|| meta))
                .filter_map(|(ix, logs)| {
                    Some(
                        ix.parse_instruction::<I>()?
                            .map(|result_with_ix| (result_with_ix, logs)),
                    )
                })
                .collect::<Result<_, _>>()
        }
    }
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
        let EncodedConfirmedTransaction {
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
