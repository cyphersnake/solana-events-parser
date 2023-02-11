use std::{result, sync::Arc, time::Duration};

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

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ResyncOrder {
    Newest,
    Historical,
}

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
    pub commitment_config: CommitmentConfig,
    pub client: Arc<RpcClient>,
    pub pubsub_client: Arc<PubsubClient>,
    pub event_recipient: Arc<EventRecipient>,
    pub resync_duration: Duration,
    pub event_consumer: EventConsumerFn,
    pub transaction_consumer: TransactionConsumerFn,
    pub local_storage: Arc<dyn Send + Sync + storage::ResyncedTransactionsPtrStorage<Error = E>>,
    pub resync_signatures_chunk_size: Option<usize>,
    pub resync_ptr_setter: Arc<dyn Send + Sync + Fn(u64) -> BoxFuture<'static, Result<()>>>,
    pub resync_order: ResyncOrder,
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
        let listen_events = {
            let self_ref = Arc::clone(&self);
            let program_id = self.program_id.to_string();
            tokio::task::spawn(async move {
                self_ref
                    .listen_events()
                    .instrument(tracing::span!(
                        tracing::Level::TRACE,
                        "Listen Events",
                        program_id = program_id
                    ))
                    .await
            })
        };
        let resync_events = {
            let self_ref = Arc::clone(&self);
            let program_id = self.program_id.to_string();
            tokio::task::spawn(async move {
                self_ref
                    .resync_events()
                    .instrument(tracing::span!(
                        tracing::Level::TRACE,
                        "Resync Event",
                        program_id = program_id,
                    ))
                    .await
            })
        };
        listen_events
            .await?
            .inspect_err(|err| tracing::error!("Error while listen events: {err:?}"))?;
        resync_events
            .await?
            .inspect_err(|err| tracing::error!("Error while resync events: {err:?}"))?;
        Ok(())
    }

    async fn listen_events(&self) -> Result<()> {
        tracing::info!("Launching pubsub client");

        let (stream, _unsubscribe) = self
            .pubsub_client
            .logs_subscribe(
                RpcTransactionLogsFilter::Mentions(vec![self.program_id.to_string()]),
                RpcTransactionLogsConfig {
                    commitment: Some(self.commitment_config),
                },
            )
            .instrument(tracing::span!(tracing::Level::TRACE, "LogsSubscribe"))
            .await
            .inspect_err(|err| tracing::error!("Error while subs: {err:?}"))
            .map_err(|err| Error::WebsocketError(err.to_string()))?;

        let mut stream = stream.inspect(|subscription_response| {
            tracing::info!(
                "Log subscription response received, transaction hash: {}",
                subscription_response.value.signature
            );
        });
        tracing::info!("Ready to listen");
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

                if self
                    .local_storage
                    .is_transaction_registered(&self.program_id, &tx_signature)?
                {
                    continue;
                }

                match (self.event_consumer)(subscription_response.value.logs) {
                    Ok(EventConsumeResult::ConsumeSuccess) => (),
                    Ok(EventConsumeResult::TransactionNeeed) => {
                        let transaction = unwrap_or_continue!(
                            self.get_transaction_by_signature(tx_signature).await,
                            "Error while transaction requesting {err:?}"
                        );

                        if let Err(err) = (self.transaction_consumer)(
                            tx_signature,
                            transaction,
                            Arc::clone(&self.client),
                            Arc::clone(&self.event_recipient),
                        )
                        .await
                        {
                            tracing::error!("Error while transaction consuming {err:?}", err = err);
                        }
                    }
                    Err(err) => {
                        tracing::error!("Error while events consuming {:?}", err);
                        continue;
                    }
                };

                self.local_storage
                    .register_transaction(&self.program_id, &tx_signature)?;
            }
        }
    }

    async fn get_unregistered_program_transactions(
        &self,
    ) -> Result<(
        u64,
        result::Result<NonEmptyVec<SolanaSignature>, EmptyError>,
    )> {
        let (resync_last_slot, mut all_signatures) = self
            .get_signatures_for_address_with_config(
                &self.program_id,
                self.commitment_config,
                self.local_storage
                    .get_last_resynced_transaction(&self.program_id)?,
            )
            .await?;

        if self.resync_order == ResyncOrder::Historical {
            all_signatures.reverse();
        }

        Ok((
            resync_last_slot,
            NonEmptyVec::try_from(
                self.local_storage
                    .filter_unregistered_transactions(&self.program_id, &all_signatures)?,
            ),
        ))
    }

    async fn resync_events(self: &Arc<Self>) -> Result<()> {
        'resync: loop {
            tokio::time::sleep(self.resync_duration).await;
            tracing::info!("Start resync: {}", self.program_id);

            let (resync_last_slot, signatures) = unwrap_or_continue!(
                self.get_unregistered_program_transactions().await,
                "Error while get unregistered program signature: {err:?}"
            );
            let signatures = match signatures {
                Ok(non_empty_signatures) => non_empty_signatures,
                Err(EmptyError) => {
                    (self.resync_ptr_setter)(resync_last_slot).await?;
                    continue 'resync;
                }
            };

            // If any of tx in resync batch failed, then not move last resync transaction pointer
            let mut last_transaction = match self.resync_order {
                ResyncOrder::Newest => Some(signatures.first()),
                ResyncOrder::Historical => Some(signatures.last()),
            }
            .copied();

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
                            "Unprocessed transaction find while resynchronization process, transaction hash: {}",
                            tx_signature.to_string()
                        );

                        let transaction = unwrap_or_continue!(
                            self_clone.get_transaction_by_signature(tx_signature).await,
                            error_action = last_transaction.take(),
                            "Error while get transaction by signature: {err:?}"
                        );

                        if let Err(err) = (self_clone.transaction_consumer)(
                            tx_signature,
                            transaction,
                            Arc::clone(&self_clone.client),
                            Arc::clone(&self_clone.event_recipient),
                        )
                        .await
                        {
                            tracing::error!("Error while transaction consuming {err:?}", err = err);
                        } else {
                            tracing::info!("Transaction {} consumed", tx_signature);
                        }

                        self_clone
                            .local_storage
                            .register_transaction(&self_clone.program_id, &tx_signature)?;
                        }

                    Result::Ok(())
                }
                    .instrument(tracing::span!(
                        tracing::Level::TRACE,
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
                continue 'resync;
            }

            if let Some(last_transaction) = last_transaction {
                self.local_storage
                    .set_last_resynced_transaction(&self.program_id, &last_transaction)?;
            }
            (self.resync_ptr_setter)(resync_last_slot).await?;
        }
    }

    async fn get_transaction_by_signature(
        &self,
        tx_signature: SolanaSignature,
    ) -> Result<TransactionParsedMeta> {
        self.client
            .bind_transaction_instructions_logs(tx_signature)
            .await
            .map_err(Error::EventParserError)
    }

    async fn get_signatures_for_address_with_config(
        &self,
        address: &Pubkey,
        commitment_config: CommitmentConfig,
        until: Option<SolanaSignature>,
    ) -> Result<(u64, Vec<SolanaSignature>)> {
        let mut all_signatures = vec![];
        let mut before = None;

        // TODO Change to batch request
        let resync_last_slot = self
            .client
            .get_slot_with_commitment(commitment_config)
            .await
            .map_err(|err| {
                tracing::error!("Error while get resync last slot: {:?}", err);
                Error::ClientError(err)
            })?;

        loop {
            tracing::trace!(
                "Request signature batch, before: {:?}, until: {:?}",
                before,
                until
            );

            let signatures_batch = self
                .client
                .get_signatures_for_address_with_config(
                    address,
                    solana_client::rpc_client::GetConfirmedSignaturesForAddress2Config {
                        before,
                        until,
                        limit: None,
                        commitment: Some(commitment_config),
                    },
                )
                .await
                .map_err(|err| {
                    tracing::error!(
                        "Error while get signature for address with config: {:?}",
                        err
                    );
                    Error::ClientError(err)
                })?
                .into_iter()
                .filter(|tx| tx.err.is_none())
                .map(|tx| {
                    tx.signature.parse().map_err(
                        |err: solana_sdk::signature::ParseSignatureError| {
                            Error::SignatureParsingError(err.to_string())
                        },
                    )
                })
                .collect::<Result<Vec<_>>>()?;

            if signatures_batch.is_empty() {
                break;
            }

            before = signatures_batch.last().copied();

            all_signatures = [signatures_batch, all_signatures].concat();
        }
        all_signatures.reverse();

        Ok((resync_last_slot, all_signatures))
    }
}
