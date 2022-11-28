use std::io;

pub use anchor_lang::{AnchorDeserialize, Discriminator, Owner};
pub use solana_sdk::{instruction::Instruction, pubkey::Pubkey};

pub use crate::{
    log_parser::ProgramLog,
    transaction_parser::{Error, TransactionParsedMeta},
};

const DISCRIMINATOR_SIZE: usize = 8;

pub trait ParseEvent {
    fn parse_event<T: Discriminator + Owner + AnchorDeserialize>(
        &self,
        program_id: Pubkey,
    ) -> Option<Result<T, io::Error>>;
}
impl ParseEvent for ProgramLog {
    fn parse_event<E: Discriminator + Owner + AnchorDeserialize>(
        &self,
        program_id: Pubkey,
    ) -> Option<Result<E, io::Error>> {
        match self {
            ProgramLog::Data(log) if E::owner().eq(&program_id) => {
                let bytes = base64::decode(log)
                    .map_err(|_| tracing::warn!("Provided log line not decodable as bs64"))
                    .ok()
                    .filter(|bytes| bytes.len() >= DISCRIMINATOR_SIZE)?;
                let (discriminantor, event) = bytes.split_at(DISCRIMINATOR_SIZE);
                E::discriminator()
                    .eq(discriminantor)
                    .then(|| E::try_from_slice(event))
            }
            _ => None,
        }
    }
}
