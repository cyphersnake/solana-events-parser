use std::{
    result,
    sync::{Arc, RwLock},
    time::Duration,
};

use async_trait::async_trait;
use futures::{future::BoxFuture, StreamExt};
use non_empty_vec::{EmptyError, NonEmpty as NonEmptyVec};
use result_inspect::ResultInspectErr;
use solana_client::{
    nonblocking::{pubsub_client::PubsubClient, rpc_client::RpcClient},
    rpc_config::{RpcTransactionLogsConfig, RpcTransactionLogsFilter},
};
use solana_sdk::commitment_config::CommitmentConfig;
use tracing::Instrument;

pub use crate::transaction_parser::{Pubkey, Signature as SolanaSignature};
use crate::{
    storage,
    transaction_parser::{BindTransactionInstructionLogs, TransactionParsedMeta},
};

macro_rules! unwrap_or_continue {
        ($result:expr) => {
            match $result {
                Ok(ok) => ok,
                Err(_err) => {
                    continue;
                }
            }
        };
        ($result:expr, $($log:tt),+ ) => {
            match $result {
                Ok(ok) => ok,
                Err(err) => {
                    tracing::error!($( $log, )* err = err);
                    continue;
                }
            }
        };
        ($result:expr, error_action = $action:expr, $($log:tt),+ ) => {
            match $result {
                Ok(ok) => ok,
                Err(err) => {
                    tracing::error!($( $log, )* err = err);
                    $action;
                    continue;
                }
            }
        };
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    TokioJoinError(#[from] tokio::task::JoinError),
    #[error(transparent)]
    EventParserError(#[from] crate::transaction_parser::Error),
    #[error("Signature parsing error: {0}")]
    SignatureParsingError(String),
    #[error("Websocket error: {0}")]
    WebsocketError(String),
    #[error(transparent)]
    ClientError(#[from] solana_client::client_error::ClientError),
    #[error("Error while use storage: {0}")]
    StorageError(String),
    #[error(transparent)]
    Client(#[from] de_solana_client::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

#[async_trait]
pub trait PassEvent {
    type Error;
    async fn pass_event(&self, raw_event: Vec<u8>) -> result::Result<(), Self::Error>;
}

pub enum EventConsumeResult {
    ConsumeSuccess,
    TransactionNeeed,
}
pub type Event = Vec<String>;
pub type EventConsumerFn = fn(Event) -> Result<EventConsumeResult>;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum ResyncOrder {
    Newest,
    Historical,
}

#[derive(derive_builder::Builder)]
pub struct EventsReader<TransactionConsumerFn, EventRecipient, E>
where
    EventRecipient: PassEvent + Send + Sync + 'static,
    TransactionConsumerFn: Send
        + Sync
        + Fn(
            SolanaSignature,
            TransactionParsedMeta,
            Arc<RpcClient>,
            Arc<EventRecipient>,
        ) -> BoxFuture<'static, Result<()>>,
    E: 'static + Send + Sync,
    Error: From<E>,
{
    pub program_id: Pubkey,
    #[builder(default = "CommitmentConfig::finalized()")]
    pub commitment_config: CommitmentConfig,
    pub client: Arc<RpcClient>,
    pub pubsub_client: Arc<PubsubClient>,
    pub event_recipient: Arc<EventRecipient>,
    #[builder(default = "Duration::from_secs(5)")]
    pub resync_duration: Duration,
    pub event_consumer: EventConsumerFn,
    pub transaction_consumer: TransactionConsumerFn,
    pub local_storage: Arc<dyn Send + Sync + storage::ResyncedTransactionsPtrStorage<Error = E>>,
    pub resync_signatures_chunk_size: Option<usize>,
    pub resync_ptr_setter: Arc<dyn Send + Sync + Fn(u64) -> BoxFuture<'static, Result<()>>>,
    pub resync_order: ResyncOrder,
    #[builder(default = "Arc::new(RwLock::new(None))")]
    pub resync_rollback: Arc<RwLock<Option<SolanaSignature>>>,
}

impl<TransactionConsumerFn, EventRecipient, E>
    EventsReader<TransactionConsumerFn, EventRecipient, E>
where
    EventRecipient: PassEvent + Send + Sync + 'static,
    TransactionConsumerFn: 'static
        + Send
        + Sync
        + Fn(
            SolanaSignature,
            TransactionParsedMeta,
            Arc<RpcClient>,
            Arc<EventRecipient>,
        ) -> BoxFuture<'static, Result<()>>,
    E: 'static + Send + Sync,
    Error: From<E>,
{
    pub async fn run(self: Arc<Self>) -> Result<()> {
        let mut tasks = tokio::task::JoinSet::default();
        let self_ref = Arc::clone(&self);
        let program_id = self.program_id.to_string();
        tasks.spawn(async move {
            self_ref
                .listen_events()
                .instrument(tracing::span!(
                    tracing::Level::ERROR,
                    "Listen Events",
                    program_id = program_id
                ))
                .await
        });
        let self_ref = Arc::clone(&self);
        let program_id = self.program_id.to_string();
        tasks.spawn(async move {
            self_ref
                .resync_events()
                .instrument(tracing::span!(
                    tracing::Level::ERROR,
                    "Resync Event",
                    program_id = program_id,
                ))
                .await
        });

        while let Some(result) = tasks.join_next().await {
            result
                .inspect_err(|err| tracing::error!("Error while join task: {err:?}"))?
                .inspect_err(|err| tracing::error!("Error while task: {err:?}"))?;
        }
        Ok(())
    }

    async fn listen_events(&self) -> Result<()> {
        tracing::info!("Launching websocket client");

        let (stream, _unsubscribe) = self
            .pubsub_client
            .logs_subscribe(
                RpcTransactionLogsFilter::Mentions(vec![self.program_id.to_string()]),
                RpcTransactionLogsConfig {
                    commitment: Some(self.commitment_config),
                },
            )
            .instrument(tracing::span!(tracing::Level::ERROR, "LogsSubscribe"))
            .await
            .inspect_err(|err| tracing::error!("Error while subs: {err:?}"))
            .map_err(|err| Error::WebsocketError(err.to_string()))?;

        let mut stream = stream.inspect(|subscription_response| {
            tracing::info!(
                "Log subscription response received, transaction hash: {}",
                subscription_response.value.signature
            );
        });
        tracing::info!("Start listening websocket events");
        loop {
            if let Some(subscription_response) = stream.next().await {
                let tx_signature = unwrap_or_continue!(
                    subscription_response
                        .value
                        .signature
                        .parse::<SolanaSignature>()
                        .map_err(|err: solana_sdk::signature::ParseSignatureError| {
                            Error::SignatureParsingError(err.to_string())
                        }),
                    "Error while tx signature parsing: {err:?}"
                );

                {
                    if self
                        .local_storage
                        .is_transaction_registered(&self.program_id, &tx_signature)?
                    {
                        tracing::info!(
                            "Transaction {tx_signature} already registered in event-parser, skip"
                        );
                        continue;
                    }

                    tracing::info!("Transaction {tx_signature} not registered yet, processing");

                    match (self.event_consumer)(subscription_response.value.logs) {
                        Ok(EventConsumeResult::ConsumeSuccess) => {
                            tracing::info!("Transaction {tx_signature} consumed successful by ws information only");
                        }
                        Ok(EventConsumeResult::TransactionNeeed) => {
                            tracing::info!("Transaction {tx_signature} direct RPC request needed");
                            let transaction = unwrap_or_continue!(
                                self.get_transaction_by_signature(tx_signature).await,
                                "Error while transaction {tx_signature} requesting {err:?}"
                            );

                            let transaction_str = tx_signature.to_string();
                            if let Err(err) = (self.transaction_consumer)(
                                tx_signature,
                                transaction,
                                Arc::clone(&self.client),
                                Arc::clone(&self.event_recipient),
                            )
                            .instrument(tracing::span!(
                                tracing::Level::ERROR,
                                "Consume",
                                tx_signature = transaction_str
                            ))
                            .await
                            {
                                tracing::error!(
                                    "Error while transaction {transaction_str} consuming {err:?}",
                                    err = err
                                );
                            } else {
                                tracing::info!(
                                    "Transaction {transaction_str} consumed as part of websocket listener",
                                );
                            }
                        }
                        Err(err) => {
                            tracing::error!(
                                "Error while events consuming {err:?} of {tx_signature}"
                            );
                            continue;
                        }
                    };

                    self.local_storage
                        .register_transaction(&self.program_id, &tx_signature)?;
                }
            }
        }
    }

    async fn get_unregistered_program_transactions(
        &self,
    ) -> Result<(
        u64,
        result::Result<NonEmptyVec<SolanaSignature>, EmptyError>,
        Option<SolanaSignature>,
    )> {
        use de_solana_client::GetTransactionsSignaturesForAddress;

        let resync_last_slot = self.client.get_slot().await?;
        let resync_start = self
            .local_storage
            .get_last_resynced_transaction(&self.program_id)?;
        tracing::info!(
            "Resync start from {}",
            resync_start
                .as_ref()
                .map(|tx| format!("{tx} transaction"))
                .unwrap_or("beginning".to_owned())
        );
        let all_signatures = <RpcClient as GetTransactionsSignaturesForAddress>::get_signatures_data_for_address_with_config(
                &self.client,
                &self.program_id,
                self.commitment_config,
                resync_start
            )
            .await?;

        // If any of tx in resync batch failed, then not move last resync transaction pointer
        let last_transaction = all_signatures.first().map(|d| d.signature);

        let all_signatures: Vec<SolanaSignature> = if self.resync_order == ResyncOrder::Historical {
            all_signatures
                .into_iter()
                .filter_map(|d| d.err.is_none().then_some(d.signature))
                .rev()
                .collect()
        } else {
            all_signatures
                .into_iter()
                .filter_map(|d| d.err.is_none().then_some(d.signature))
                .collect()
        };

        Ok((
            resync_last_slot,
            NonEmptyVec::try_from(
                self.local_storage
                    .filter_unregistered_transactions(&self.program_id, &all_signatures)?,
            ),
            last_transaction,
        ))
    }

    async fn resync_events(self: &Arc<Self>) -> Result<()> {
        'resync: loop {
            tokio::time::sleep(self.resync_duration).await;
            tracing::info!("Start resync for program {}", self.program_id);

            let (resync_last_slot, signatures, mut last_transaction) = unwrap_or_continue!(
                self.get_unregistered_program_transactions().await,
                "Error while get unregistered program signature: {err:?}"
            );
            let signatures = match signatures {
                Ok(non_empty_signatures) => non_empty_signatures,
                Err(EmptyError) => {
                    (self.resync_ptr_setter)(resync_last_slot).await?;
                    if let Some(last_transaction) = last_transaction {
                        self.set_last_resynced_transaction(&last_transaction)?;
                    }
                    tracing::info!("Resync ended: no new transactions");
                    continue 'resync;
                }
            };

            tracing::info!(
                "Find new {} transactions, start processing",
                signatures.len()
            );

            let signatures_chunks = signatures
                .as_slice()
                .chunks(
                    self.resync_signatures_chunk_size
                        .unwrap_or_else(|| signatures.len().get()),
                )
                .enumerate();

            let mut tasks = tokio::task::JoinSet::new();
            for (index, signatures_chunk) in signatures_chunks {
                let self_clone = self.clone();
                let signatures_chunk = signatures_chunk.to_vec();

                tasks.spawn(async move {
                    for tx_signature in signatures_chunk.into_iter() {
                        tracing::info!(
                            "Unprocessed (by ws) transaction find while resynchronization process, transaction hash: {}",
                            tx_signature.to_string()
                        );

                        let transaction = unwrap_or_continue!(
                            self_clone.get_transaction_by_signature(tx_signature).await,
                            error_action = last_transaction.take(),
                            "Error while get transaction by signature: {err:?}"
                        );

                        let transaction_str = tx_signature.to_string();
                        if let Err(err) = (self_clone.transaction_consumer)(
                            tx_signature,
                            transaction,
                            Arc::clone(&self_clone.client),
                            Arc::clone(&self_clone.event_recipient),
                        )
                        .await
                        {
                            tracing::error!("Error while transaction {transaction_str} consuming {err:?}", err = err);
                        } else {
                            tracing::info!("Transaction {tx_signature} consumed as part of resync process");
                        }

                        self_clone
                            .local_storage
                            .register_transaction(&self_clone.program_id, &tx_signature)?;
                        }

                    Result::Ok(())
                }
                    .instrument(tracing::span!(
                        tracing::Level::ERROR,
                        "Register chunk",
                        chunk_index = index,
                    ))
                );
            }

            let mut tasks_success = true;
            while let Some(task) = tasks.join_next().await {
                tasks_success &= match task {
                    Ok(Ok(())) => true,
                    Ok(Err(err)) => {
                        tracing::error!("Error while resync task: {err:?}");
                        false
                    }
                    Err(err) => {
                        tracing::error!("Error while join resync task: {err:?}");
                        false
                    }
                };
            }

            if !tasks_success {
                tracing::warn!("Some of resync tasks failed, not move resync ptr");
                continue 'resync;
            }

            if let Some(last_transaction) = last_transaction {
                tracing::warn!("resync successful ended, ptr moved to {last_transaction}");
                self.set_last_resynced_transaction(&last_transaction)?;
            } else {
                tracing::warn!("resync successful ended, not new ptr for move");
            }

            (self.resync_ptr_setter)(resync_last_slot).await?;
        }
    }

    fn set_last_resynced_transaction(
        self: &Arc<Self>,
        last_transaction: &SolanaSignature,
    ) -> Result<()> {
        let last_transaction = self
            .resync_rollback
            .write()
            .ok()
            .and_then(|mut write| {
                write.take().map(|tx| {
                    tracing::info!("Found rollback to {tx} transaction");
                    tx
                })
            })
            .unwrap_or(*last_transaction);

        tracing::info!("Set last resynced tx to {last_transaction} transaction");
        self.local_storage
            .set_last_resynced_transaction(&self.program_id, &last_transaction)?;

        Ok(())
    }

    async fn get_transaction_by_signature(
        &self,
        tx_signature: SolanaSignature,
    ) -> Result<TransactionParsedMeta> {
        self.client
            .bind_transaction_instructions_logs(tx_signature, self.commitment_config)
            .await
            .map_err(Error::EventParserError)
    }
}
