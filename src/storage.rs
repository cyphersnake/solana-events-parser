//! This module is used to help the [`crate::event_reader_service`] storage management
//! It allows us to keep track of which transactions have already been processed
//! (registered) and store a pointer to the transaction - resync boundary

use std::fmt;

pub use crate::transaction_parser::{Pubkey, Signature as SolanaSignature};

/// [`RegisterTransaction`] is a trait for managing transactions.
///
/// It provides methods for registering a transaction, checking if a transaction is registered,
/// and filtering unregistered transactions.
pub trait RegisterTransaction {
    type Error: fmt::Debug;

    /// Register a transaction with the given `program_id` and `transaction_hash`.
    fn register_transaction(
        &self,
        program_id: &Pubkey,
        transaction_hash: &SolanaSignature,
    ) -> Result<(), Self::Error>;

    /// Check if a transaction with the given `program_id` and `transaction_hash` is registered
    /// with [`RegisterTransaction::register_transaction`] before
    fn is_transaction_registered(
        &self,
        program_id: &Pubkey,
        transaction_hash: &SolanaSignature,
    ) -> Result<bool, Self::Error>;

    /// Given a `program_id` and a list of transactions (`transaction_hash_set`),
    /// filter out those that are not registered.
    ///
    /// Returns a `Result` containing a `Vec` of unregistered `SolanaSignature`
    fn filter_unregistered_transactions(
        &self,
        program_id: &Pubkey,
        transaction_hash_set: &[SolanaSignature],
    ) -> Result<Vec<SolanaSignature>, Self::Error>;
}

/// This trait extends [`RegisterTransaction`]
/// and provides methods for managing the last resynced transaction.
pub trait ResyncedTransactionsPtrStorage: RegisterTransaction {
    /// Initializes the last resynced transaction if it's not initialized before.
    fn initialize_if_needed_resynced_transaction(
        &self,
        program_id: &Pubkey,
        transaction: &SolanaSignature,
    ) -> Result<(), <Self as RegisterTransaction>::Error>;

    /// Get last resynced transaction, initialized
    /// by [`ResyncedTransactionsPtrStorage::initialize_if_needed_resynced_transaction`] or
    /// setted by [`ResyncedTransactionsPtrStorage::set_last_resynced_transaction`]
    fn get_last_resynced_transaction(
        &self,
        program_id: &Pubkey,
    ) -> Result<Option<SolanaSignature>, <Self as RegisterTransaction>::Error>;

    /// Set last recyned transaction into new one
    fn set_last_resynced_transaction(
        &self,
        program_id: &Pubkey,
        transaction: &SolanaSignature,
    ) -> Result<(), <Self as RegisterTransaction>::Error>;
}

#[cfg(feature = "rocksdb")]
pub mod rocksdb {
    use rocksdb::{DBWithThreadMode, MultiThreaded};

    use super::{Pubkey, RegisterTransaction, ResyncedTransactionsPtrStorage, SolanaSignature};

    #[derive(Debug)]
    pub enum Error {
        RocksDb(rocksdb::Error),
        Bincode(bincode::Error),
    }
    impl From<rocksdb::Error> for Error {
        fn from(err: rocksdb::Error) -> Self {
            Self::RocksDb(err)
        }
    }
    impl From<bincode::Error> for Error {
        fn from(err: bincode::Error) -> Self {
            Error::Bincode(err)
        }
    }
    #[cfg(feature = "event-reader")]
    impl From<Error> for crate::event_reader_service::Error {
        fn from(error: Error) -> Self {
            Self::StorageError(format!("{error:?}"))
        }
    }

    pub type DB = DBWithThreadMode<MultiThreaded>;

    fn construct_key(program_id: &Pubkey, transaction_hash: &SolanaSignature) -> Vec<u8> {
        [
            KEY_SUFFIX,
            program_id.to_bytes().as_ref(),
            transaction_hash.as_ref(),
        ]
        .concat()
    }

    const LAST_RESYNCED_SUFFIX: &[u8] = b"_last_resynced";
    const KEY_SUFFIX: &[u8] = b"tx";

    impl RegisterTransaction for DB {
        type Error = Error;

        fn register_transaction(
            &self,
            program_id: &Pubkey,
            transaction_hash: &SolanaSignature,
        ) -> Result<(), Self::Error> {
            self.put(construct_key(program_id, transaction_hash), [])?;
            Ok(())
        }

        fn is_transaction_registered(
            &self,
            program_id: &Pubkey,
            transaction_hash: &SolanaSignature,
        ) -> Result<bool, Self::Error> {
            Ok(self
                .get(construct_key(program_id, transaction_hash))?
                .is_some())
        }

        fn filter_unregistered_transactions(
            &self,
            program_id: &Pubkey,
            transaction_hash_set: &[SolanaSignature],
        ) -> Result<Vec<SolanaSignature>, Self::Error> {
            self.multi_get(
                transaction_hash_set
                    .iter()
                    .map(|tx| construct_key(program_id, tx)),
            )
            .into_iter()
            .zip(transaction_hash_set.iter())
            .try_fold(vec![], |mut accum, (result, transaction_hash)| {
                if result?.is_none() {
                    accum.push(*transaction_hash);
                }
                Ok(accum)
            })
        }
    }

    impl ResyncedTransactionsPtrStorage for DB {
        fn initialize_if_needed_resynced_transaction(
            &self,
            program_id: &Pubkey,
            transaction: &SolanaSignature,
        ) -> Result<(), <Self as RegisterTransaction>::Error> {
            // FIXME: remove non-atomic set
            if self.get_last_resynced_transaction(program_id)?.is_none() {
                self.set_last_resynced_transaction(program_id, transaction)?;
            }
            Ok(())
        }

        fn get_last_resynced_transaction(
            &self,
            program_id: &Pubkey,
        ) -> Result<Option<SolanaSignature>, Self::Error> {
            Ok(self
                .get([&program_id.to_bytes()[..], LAST_RESYNCED_SUFFIX].concat())?
                .map(|raw| bincode::deserialize(&raw))
                .transpose()?)
        }

        fn set_last_resynced_transaction(
            &self,
            program_id: &Pubkey,
            transaction: &SolanaSignature,
        ) -> Result<(), Self::Error> {
            self.put(
                [&program_id.to_bytes()[..], LAST_RESYNCED_SUFFIX].concat(),
                bincode::serialize(&transaction)?,
            )?;

            Ok(())
        }
    }
}
