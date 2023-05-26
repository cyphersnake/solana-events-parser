use std::io;

pub use anchor_lang::{AnchorDeserialize, Discriminator, Owner};
pub use solana_sdk::{instruction::Instruction, pubkey::Pubkey};

pub use crate::{
    log_parser::ProgramLog,
    transaction_parser::{Error, TransactionParsedMeta},
};

const DISCRIMINATOR_SIZE: usize = 8;

/// [`ParseEvent`] is a trait providing the method [`ParseEvent::parse_event`] to parse events
/// from the [`crate::log_parser::ProgramLog`].
///
/// The trait is defined for any type `T` that implements:
/// - [`anchor_lang::Discriminator`] - Defines a specific event type via a binary prefix
/// - [`anchor_lang::Owner`] - Links the type of event and its "owner" (solana-program)
/// - [`anchor_lang::AnchorDeserialize`] - Enables events to be deserialised in the structure
///
/// For the debridge-finance anchor fork, these traits are defined for all events, however,
/// if you wish to use the original anchor, you will need to manually implement some of
/// these traits.
///
/// ```
/// use solana_events_parser::{ParseEvent, log_parser::ProgramLog};
///
/// use anchor_lang::prelude::*;
///
/// const PROGRAM_ID: Pubkey = Pubkey::new_from_array([0; 32]);
///
/// #[derive(anchor_lang::AnchorDeserialize)]
/// struct Event;
///
/// impl anchor_lang::Owner for Event {
///     fn owner() -> Pubkey {
///         PROGRAM_ID
///     }
/// }
/// impl anchor_lang::Discriminator for Event {
///     const DISCRIMINATOR: [u8; 8] = [1u8; 8];
/// }
///
/// let event = ProgramLog::Data("anVzdCBhIGV4YW1wbGUsIHdoYXQgeW91IGV4cGVjdGVkPw==".to_owned())
///     .parse_event::<Event>(PROGRAM_ID);
/// ```
///
/// The `parse_event` method takes a `program_id` and returns an `Option` which will be `None` if no event
/// was parsed and `Some` with a `Result` containing either the parsed event or an error.
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
