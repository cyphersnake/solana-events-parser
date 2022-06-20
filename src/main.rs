#[cfg(feature = "anchor")]
pub mod event_parser;
#[cfg(feature = "solana")]
pub mod instruction_parser;
#[cfg(feature = "solana")]
pub mod transaction_parser;

pub mod log_parser;

use anyhow::anyhow;
use simple_logger::SimpleLogger;

#[cfg(feature = "anchor")]
pub use crate::{event_parser::ParseEvent, instruction_parser::ParseInstruction};
#[cfg(feature = "solana")]
pub use transaction_parser::{BindTransactionInstructionLogs, BindTransactionLogs};

fn main() -> Result<(), anyhow::Error> {
    SimpleLogger::new()
        .env()
        .init()
        .map_err(|err| anyhow!("Error while init logger: {}", err))?;

    #[cfg(feature = "solana")]
    {
        use std::{env, str::FromStr};

        use transaction_parser::*;

        let events = RpcClient::new("https://api.mainnet-beta.solana.com")
            .bind_transaction_instructions_logs(
                Signature::from_str(&env::args().nth(1).ok_or_else(|| {
                    anyhow!(
                    "Signatures not provided, Use first argument for provide transaction signature"
                )
                })?)
                .map_err(|err| {
                    anyhow!(
                        "Error while parsing argument as transaction signature: {}",
                        err
                    )
                })?,
            )
            .map_err(|err| anyhow!("Error while bind transaction instructions: {}", err))?
            .meta;

        println!(
            "{}",
            serde_json::to_string_pretty(&events.into_iter().collect::<Vec<_>>())
                .map_err(|err| { anyhow!("Error while serialize result of binding: {}", err) })?
        );
    }

    #[cfg(not(feature = "solana"))]
    println!("No action when solana feature disable");

    Ok(())
}
