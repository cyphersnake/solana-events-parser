use std::io;

use anchor_lang::{AnchorDeserialize, Discriminator, Owner};

pub use solana_sdk::instruction::Instruction;

pub use crate::log_parser::ProgramLog;

const DISCRIMINATOR_SIZE: usize = 8;

pub trait ParseEvent {
    fn parse_event<T: Discriminator + AnchorDeserialize>(&self) -> Option<Result<T, io::Error>>;
}
impl ParseEvent for ProgramLog {
    fn parse_event<E: Discriminator + AnchorDeserialize>(&self) -> Option<Result<E, io::Error>> {
        match self {
            ProgramLog::Data(log) => {
                let bytes = base64::decode(&log)
                    .map_err(|_| log::warn!("Provided log line not decodable as bs64"))
                    .ok()
                    .filter(|bytes| bytes.len() > DISCRIMINATOR_SIZE)?;
                let (discriminantor, event) = bytes.split_at(DISCRIMINATOR_SIZE);
                E::discriminator()
                    .eq(discriminantor)
                    .then(|| E::try_from_slice(event))
            }
            _ => None,
        }
    }
}

pub trait ParseInstruction {
    fn parse_instruction<T: Discriminator + Owner + AnchorDeserialize>(
        &self,
    ) -> Option<Result<T, io::Error>>;
}
impl ParseInstruction for Instruction {
    fn parse_instruction<I: Discriminator + Owner + AnchorDeserialize>(
        &self,
    ) -> Option<Result<I, io::Error>> {
        let (discriminantor, event) = self.data.split_at(DISCRIMINATOR_SIZE);
        (I::owner().eq(&self.program_id) && I::discriminator().eq(discriminantor))
            .then(|| I::try_from_slice(event))
    }
}
