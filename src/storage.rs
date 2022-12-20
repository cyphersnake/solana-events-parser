pub use crate::transaction_parser::{Pubkey, Signature as SolanaSignature};

pub trait RegisterTransaction {
    type Error;

    fn register_transaction(
        &self,
        program_id: &Pubkey,
        transaction_hash: &SolanaSignature,
    ) -> Result<(), Self::Error>;

    fn is_transaction_registered(
        &self,
        program_id: &Pubkey,
        transaction_hash: &SolanaSignature,
    ) -> Result<bool, Self::Error>;

    fn filter_unregistered_transactions(
        &self,
        program_id: &Pubkey,
        transaction_hash_set: &[SolanaSignature],
    ) -> Result<Vec<SolanaSignature>, Self::Error>;
}

pub trait ResyncedTransactionsPtrStorage: RegisterTransaction {
    fn initialize_if_needed_resynced_transaction(
        &self,
        program_id: &Pubkey,
        transaction: &SolanaSignature,
    ) -> Result<(), <Self as RegisterTransaction>::Error>;

    fn get_last_resynced_transaction(
        &self,
        program_id: &Pubkey,
    ) -> Result<Option<SolanaSignature>, <Self as RegisterTransaction>::Error>;

    fn set_last_resynced_transaction(
        &self,
        program_id: &Pubkey,
        transaction: &SolanaSignature,
    ) -> Result<(), <Self as RegisterTransaction>::Error>;
}

#[cfg(feature = "rocksdb")]
pub mod rocksdb {
    use super::{Pubkey, RegisterTransaction, ResyncedTransactionsPtrStorage, SolanaSignature};
    use rocksdb::{DBWithThreadMode, MultiThreaded};

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
            Self::StorageError(format!("{:?}", error))
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
