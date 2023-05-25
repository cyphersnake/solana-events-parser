#![allow(unstable_name_collisions)]

/// Parse anchor based events into event structure
#[cfg(feature = "anchor")]
pub mod event_parser;

/// Bind instructions into [`HashMap<InstructionContext, (Instruction, OuterInstruction)>`]
///
/// Allows [`solana_transaction_status::EncodedTransactionWithStatusMeta`] to be broken down
/// into instructions with isolated contexts.
#[cfg(feature = "solana")]
pub mod instruction_parser;

/// Allows you to query a transaction from RPC
/// and build a [`transaction_parser::TransactionParsedMeta`] on it
#[cfg(feature = "solana")]
pub mod transaction_parser;

/// Parses logs of solana programs based on regular expressions.
pub mod log_parser;

#[cfg(feature = "solana")]
pub use crate::transaction_parser::{BindTransactionInstructionLogs, BindTransactionLogs};

#[cfg(feature = "anchor")]
pub use crate::{event_parser::ParseEvent, instruction_parser::ParseInstruction};

/// Set of abstractions for storage management used in [`event_reader_service`]
#[cfg(feature = "storage")]
pub mod storage;

/// Service for automatic interception and processing of specific pubkey transactions
#[cfg(feature = "event-reader")]
pub mod event_reader_service;

#[cfg(feature = "solana")]
pub use de_solana_client;
