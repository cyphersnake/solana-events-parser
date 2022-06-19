pub mod log_parser;
pub mod transaction_parser;

use std::{env, str::FromStr};

use anyhow::anyhow;
use simple_logger::SimpleLogger;
use transaction_parser::*;

fn main() -> Result<(), anyhow::Error> {
    SimpleLogger::new()
        .env()
        .init()
        .map_err(|err| anyhow!("Error while init logger: {}", err))?;
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
    Ok(())
}
