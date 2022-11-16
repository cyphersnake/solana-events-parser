use std::{
    collections::HashMap, fmt::Debug, io, io::ErrorKind, marker::PhantomData, num::ParseIntError,
    pin::Pin, str::FromStr, sync::Arc,
};

use anchor_lang::AnchorDeserialize;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
pub use solana_client::nonblocking::rpc_client::RpcClient;
use solana_client::rpc_config::RpcTransactionConfig;
pub use solana_sdk::{
    clock::UnixTimestamp,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::Signature,
    slot_history::Slot,
};
use solana_sdk::{commitment_config::CommitmentConfig, pubkey::ParsePubkeyError};
use solana_transaction_status::option_serializer::OptionSerializer;
pub use solana_transaction_status::{
    EncodedConfirmedTransactionWithStatusMeta, EncodedTransactionWithStatusMeta, UiInstruction,
    UiTransactionEncoding, UiTransactionTokenBalance,
};

use crate::{
    event_parser::{Discriminator, Owner},
    instruction_parser::GetLoadedAccounts,
    ParseInstruction,
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
    #[error(transparent)]
    ParsePubkeyError(#[from] ParsePubkeyError),
    #[error("Can't find ix ctx {0:?} in logs")]
    InstructionLogsConsistencyError(InstructionContext),
    #[error("Provided log and provided ix not match by owner")]
    InstructionLogsOwnerError { ix_owner: Pubkey, log_owner: Pubkey },
    #[error("Failed while transaction decoding with signature: {0}")]
    ErrorWhileDecodeTransaction(Signature),
    #[error(transparent)]
    ParseIntError(#[from] ParseIntError),
    #[error("Pre token account don't match with post")]
    WrongBalanceAccountConsistance(Pubkey),
    #[error("Wrong parser found")]
    WrongParserFound,
    #[error("Failed to consume instrucition with error msg: {0}")]
    ErrorWhileConsume(String),
}

#[async_trait]
pub trait BindTransactionLogs {
    async fn bind_transaction_logs(
        &self,
        signature: Signature,
    ) -> Result<HashMap<ProgramContext, Vec<ProgramLog>>, Error>;
}

#[async_trait]
impl BindTransactionLogs for RpcClient {
    async fn bind_transaction_logs(
        &self,
        signature: Signature,
    ) -> Result<HashMap<ProgramContext, Vec<ProgramLog>>, Error> {
        Ok(log_parser::parse_events(
            match self
                .get_transaction_with_config(
                    &signature,
                    RpcTransactionConfig {
                        encoding: Some(UiTransactionEncoding::Base58),
                        max_supported_transaction_version: Some(0),
                        commitment: Some(CommitmentConfig::finalized()),
                    },
                )
                .await?
                .transaction
                .meta
                .ok_or(Error::EmptyMetaInTransaction(signature))?
                .log_messages
            {
                OptionSerializer::Skip | OptionSerializer::None => {
                    Err(Error::EmptyLogsInTransaction(signature))
                }
                OptionSerializer::Some(some) => Ok(some),
            }?
            .as_slice(),
        )?)
    }
}

pub type AmountDiff = i128;
pub type ChildProgramContext = ProgramContext;
pub type ParentProgramContext = ProgramContext;
#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionParsedMeta {
    pub meta: HashMap<ProgramContext, (Instruction, Vec<ProgramLog>)>,
    pub slot: Slot,
    pub block_time: Option<UnixTimestamp>,
    pub lamports_changes: HashMap<Pubkey, AmountDiff>,
    pub token_balances_changes: HashMap<WalletContext, AmountDiff>,
    pub parent_ix: HashMap<ChildProgramContext, ParentProgramContext>,
}

pub struct DecomposedInstruction<IX, ACCOUNTS> {
    pub program_ctx: ProgramContext,
    pub ix: IX,
    pub accounts: ACCOUNTS,
    pub logs: Vec<ProgramLog>,
}

#[async_trait]
pub trait ConsumeInstruction {
    async fn consume_ix(self: Box<Self>) -> Result<(), Error>;
}

pub type Consumer<IX, ACCOUNTS> = Arc<
    dyn Fn(
            Box<DecomposedInstructionWithConsumer<IX, ACCOUNTS>>,
        ) -> Pin<Box<dyn futures::Future<Output = Result<(), Error>> + Send>>
        + Send
        + Sync,
>;

pub struct DecomposedInstructionWithConsumer<IX, ACCOUNTS> {
    pub decomposed_ix: DecomposedInstruction<IX, ACCOUNTS>,
    pub consumer: Option<Consumer<IX, ACCOUNTS>>,
}

#[async_trait]
impl<IX: Send + Sync, ACCOUNTS: Send + Sync> ConsumeInstruction
    for DecomposedInstructionWithConsumer<IX, ACCOUNTS>
{
    async fn consume_ix(self: Box<Self>) -> Result<(), Error> {
        if let Some(consumer) = self.consumer.as_ref() {
            (consumer.clone())(self).await?;
        }
        Ok(())
    }
}

pub trait DecomposeInstruction {
    fn is_decomposable(&self, program_ctx: &ProgramContext, raw_ix: &Instruction) -> bool;

    fn decompose_instruction(
        &self,
        program_ctx: ProgramContext,
        raw_ix: &Instruction,
        logs: &[ProgramLog],
    ) -> Result<Box<dyn ConsumeInstruction + Send>, io::Error>;
}

pub struct InstructionDecomposer<
    IX: Discriminator + Owner + AnchorDeserialize + Send,
    ACCOUNTS: From<[Pubkey; ACCOUNTS_COUNT]> + Send,
    const ACCOUNTS_COUNT: usize,
> {
    ix: PhantomData<IX>,
    accounts: PhantomData<ACCOUNTS>,
    consumer: Option<Consumer<IX, ACCOUNTS>>,
}

impl<
        IX: 'static + Discriminator + Owner + AnchorDeserialize + Send + Sync,
        ACCOUNTS: 'static + From<[Pubkey; ACCOUNTS_COUNT]> + Send + Sync,
        const ACCOUNTS_COUNT: usize,
    > InstructionDecomposer<IX, ACCOUNTS, ACCOUNTS_COUNT>
{
    pub fn new_boxed() -> Box<dyn DecomposeInstruction + Send + Sync> {
        Self::default().boxed()
    }

    pub fn boxed(self) -> Box<dyn DecomposeInstruction + Send + Sync> {
        Box::new(self)
    }

    pub fn set_consumer(mut self, consumer: Consumer<IX, ACCOUNTS>) -> Self {
        self.consumer = Some(consumer);

        self
    }
}

impl<
        IX: 'static + Discriminator + Owner + AnchorDeserialize + Send + Sync,
        ACCOUNTS: 'static + From<[Pubkey; ACCOUNTS_COUNT]> + Send + Sync,
        const ACCOUNTS_COUNT: usize,
    > Default for InstructionDecomposer<IX, ACCOUNTS, ACCOUNTS_COUNT>
{
    fn default() -> Self {
        Self {
            ix: Default::default(),
            accounts: Default::default(),
            consumer: None,
        }
    }
}

impl<
        IX: 'static + Discriminator + Owner + AnchorDeserialize + Send + Sync,
        ACCOUNTS: 'static + From<[Pubkey; ACCOUNTS_COUNT]> + Send + Sync,
        const ACCOUNTS_COUNT: usize,
    > DecomposeInstruction for InstructionDecomposer<IX, ACCOUNTS, ACCOUNTS_COUNT>
{
    fn is_decomposable(&self, program_ctx: &ProgramContext, raw_ix: &Instruction) -> bool {
        const DISCRIMINATOR_SIZE: usize = 8;
        program_ctx.program_id.eq(&IX::owner())
            && IX::owner().eq(&raw_ix.program_id)
            && IX::discriminator().eq(raw_ix.data.split_at(DISCRIMINATOR_SIZE).0)
    }

    fn decompose_instruction(
        &self,
        program_ctx: ProgramContext,
        raw_ix: &Instruction,
        logs: &[ProgramLog],
    ) -> Result<Box<dyn ConsumeInstruction + Send + 'static>, io::Error> {
        Ok(Box::new(DecomposedInstructionWithConsumer {
            consumer: self.consumer.as_ref().cloned(),
            decomposed_ix: DecomposedInstruction {
                program_ctx,
                logs: logs.to_vec(),
                accounts: ACCOUNTS::from(
                    <[Pubkey; ACCOUNTS_COUNT]>::try_from(
                        raw_ix
                            .accounts
                            .iter()
                            .map(|acc| acc.pubkey)
                            .take(ACCOUNTS_COUNT)
                            .collect::<Vec<_>>(),
                    )
                    .map_err(|err| {
                        io::Error::new(
                            ErrorKind::InvalidData,
                            format!("Instruction accounts parsing error:{:?}", err),
                        )
                    })?,
                ),
                ix: raw_ix.parse_instruction::<IX>().ok_or_else(|| {
                    io::Error::new(ErrorKind::InvalidData, Error::WrongParserFound)
                })??,
            },
        }))
    }
}

#[cfg(feature = "anchor")]
mod anchor {
    use std::{io, io::ErrorKind, sync::Arc};

    use anchor_lang::{AnchorDeserialize, Discriminator, Owner};

    use super::{Pubkey, TransactionParsedMeta};
    use crate::transaction_parser::{
        ConsumeInstruction, DecomposeInstruction, DecomposedInstruction,
    };

    impl TransactionParsedMeta {
        pub fn find_and_decompose_ix<
            const ACCOUNTS_COUNT: usize,
            IX: Discriminator + Owner + AnchorDeserialize,
            ACCOUNTS: From<[Pubkey; ACCOUNTS_COUNT]>,
        >(
            &self,
        ) -> Result<Vec<DecomposedInstruction<IX, ACCOUNTS>>, io::Error> {
            use crate::ParseInstruction;
            self.meta
                .iter()
                .filter(|(ctx, _meta)| ctx.program_id.eq(&IX::owner()))
                .filter_map(|(program_ctx, (raw_instruction, logs))| {
                    Some(
                        raw_instruction
                            .parse_instruction::<IX>()?
                            .map(|instruction| {
                                Ok(DecomposedInstruction {
                                    program_ctx: *program_ctx,
                                    logs: logs.to_vec(),
                                    accounts: ACCOUNTS::from(
                                        <[Pubkey; ACCOUNTS_COUNT]>::try_from(
                                            raw_instruction
                                                .accounts
                                                .iter()
                                                .map(|acc| acc.pubkey)
                                                .take(ACCOUNTS_COUNT)
                                                .collect::<Vec<_>>(),
                                        )
                                        .map_err(
                                            |err| {
                                                io::Error::new(
                                                    ErrorKind::InvalidData,
                                                    format!(
                                                        "Instruction accounts parsing error:{:?}",
                                                        err
                                                    ),
                                                )
                                            },
                                        )?,
                                    ),
                                    ix: instruction,
                                })
                            }),
                    )
                })
                .collect::<Result<_, _>>()?
        }

        pub fn find_and_decompose_ix_with_decomposer(
            &self,
            decomposers: Arc<Vec<Box<dyn DecomposeInstruction + Send + Sync>>>,
        ) -> Result<Vec<Box<(dyn ConsumeInstruction + Send)>>, io::Error> {
            self.meta
                .iter()
                .filter_map(|(program_ctx, (raw_instruction, logs))| {
                    decomposers
                        .iter()
                        .find(|decomposer| decomposer.is_decomposable(program_ctx, raw_instruction))
                        .map(|decomposer| {
                            decomposer.decompose_instruction(*program_ctx, raw_instruction, logs)
                        })
                })
                .collect::<Result<Vec<_>, _>>()
        }
    }
}

#[async_trait]
pub trait BindTransactionInstructionLogs {
    async fn bind_transaction_instructions_logs(
        &self,
        signature: Signature,
    ) -> Result<TransactionParsedMeta, Error>;
}

#[async_trait]
impl BindTransactionInstructionLogs for RpcClient {
    async fn bind_transaction_instructions_logs(
        &self,
        signature: Signature,
    ) -> Result<TransactionParsedMeta, Error> {
        let EncodedConfirmedTransactionWithStatusMeta {
            transaction,
            slot,
            block_time,
        } = self
            .get_transaction_with_config(
                &signature,
                RpcTransactionConfig {
                    encoding: Some(UiTransactionEncoding::Base58),
                    max_supported_transaction_version: Some(0),
                    commitment: Some(CommitmentConfig::finalized()),
                },
            )
            .await?;
        let mut instructions = transaction.bind_instructions(signature)?;

        let meta = transaction
            .meta
            .as_ref()
            .ok_or(Error::EmptyMetaInTransaction(signature))?;

        let meta: HashMap<ProgramContext, (Instruction, Vec<ProgramLog>)> =
            log_parser::parse_events(match meta.log_messages.as_ref() {
                OptionSerializer::None | OptionSerializer::Skip => {
                    Err(Error::EmptyLogsInTransaction(signature))
                }
                OptionSerializer::Some(log_messages) => Ok(log_messages.as_slice()),
            }?)?
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
            .collect::<Result<_, Error>>()?;

        Ok(TransactionParsedMeta {
            slot,
            block_time,
            parent_ix: meta
                .iter()
                .flat_map(|(parent_ctx, (_, program_logs))| {
                    program_logs
                        .iter()
                        .filter_map(|program_log| match program_log {
                            ProgramLog::Invoke(children_ctx) => Some((*children_ctx, *parent_ctx)),
                            _ => None,
                        })
                })
                .collect(),
            meta,
            lamports_changes: transaction.get_lamports_changes(&signature)?,
            token_balances_changes: transaction.get_assets_changes(&signature)?,
        })
    }
}

pub trait GetLamportsChanges {
    fn get_lamports_changes(
        &self,
        signature: &Signature,
    ) -> Result<HashMap<Pubkey, AmountDiff>, Error>;
}
impl GetLamportsChanges for EncodedTransactionWithStatusMeta {
    fn get_lamports_changes(
        &self,
        signature: &Signature,
    ) -> Result<HashMap<Pubkey, AmountDiff>, Error> {
        let loaded_accounts = self
            .get_loaded_accounts()
            .ok_or(Error::ErrorWhileDecodeTransaction(*signature))??;

        let meta = self
            .meta
            .as_ref()
            .ok_or(Error::EmptyMetaInTransaction(*signature))?;

        Ok(meta
            .pre_balances
            .iter()
            .zip(meta.post_balances.iter())
            .enumerate()
            .map(|(index, (old_balance, new_balance))| {
                (index, *new_balance as i128 - *old_balance as i128)
            })
            .map(|(index, diff)| (loaded_accounts[index], diff))
            .collect())
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct WalletContext {
    pub wallet_address: Pubkey,
    pub wallet_owner: Option<Pubkey>,
    pub token_mint: Pubkey,
}
impl WalletContext {
    fn try_new(balance: &UiTransactionTokenBalance, accounts: &[Pubkey]) -> Result<Self, Error> {
        Ok(WalletContext {
            wallet_address: accounts[balance.account_index as usize],
            wallet_owner: match &balance.owner {
                OptionSerializer::None | OptionSerializer::Skip => None,
                OptionSerializer::Some(owner) => Some(Pubkey::from_str(owner)),
            }
            .transpose()?,
            token_mint: Pubkey::from_str(balance.mint.as_str())?,
        })
    }
}

pub trait GetAssetsChanges {
    fn get_assets_changes(
        &self,
        signature: &Signature,
    ) -> Result<HashMap<WalletContext, AmountDiff>, Error>;
}
impl GetAssetsChanges for EncodedTransactionWithStatusMeta {
    fn get_assets_changes(
        &self,
        signature: &Signature,
    ) -> Result<HashMap<WalletContext, AmountDiff>, Error> {
        let loaded_accounts = self
            .get_loaded_accounts()
            .ok_or(Error::ErrorWhileDecodeTransaction(*signature))??;

        let meta = self
            .meta
            .as_ref()
            .ok_or(Error::EmptyMetaInTransaction(*signature))?;

        let try_parse_balance = |balance: &UiTransactionTokenBalance| {
            Ok((
                WalletContext::try_new(balance, &loaded_accounts)?,
                balance.ui_token_amount.amount.parse()?,
            ))
        };

        let pre_token_balances = match &meta.pre_token_balances {
            OptionSerializer::Some(pre_token_balances) => Some(pre_token_balances),
            OptionSerializer::None | OptionSerializer::Skip => None,
        };
        let post_token_balances = match &meta.post_token_balances {
            OptionSerializer::Some(post_token_balances) => Some(post_token_balances),
            OptionSerializer::None | OptionSerializer::Skip => None,
        };

        pre_token_balances
            .zip(post_token_balances)
            .map(|(pre_token_balances, post_token_balances)| {
                let balances_diff = post_token_balances
                    .iter()
                    .map(try_parse_balance)
                    .collect::<Result<HashMap<_, _>, Error>>()?;

                pre_token_balances.iter().map(try_parse_balance).try_fold(
                    balances_diff,
                    |mut balances_diff, result_with_ctx| {
                        let (wallet_ctx, pre_balance) = result_with_ctx?;

                        *balances_diff.entry(wallet_ctx).or_insert(0) -= pre_balance;

                        Ok(balances_diff)
                    },
                )
            })
            .unwrap_or_else(|| Ok(HashMap::default()))
    }
}
