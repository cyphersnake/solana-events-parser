use std::{collections::HashMap, fmt::Debug, num::NonZeroU8, str::FromStr};

use lazy_static::lazy_static;
#[cfg(not(feature = "solana"))]
pub use pubkey::Pubkey;
use regex::Regex;
use serde::{Deserialize, Serialize};
#[cfg(feature = "solana")]
pub use solana_sdk::pubkey::Pubkey;

lazy_static! {
    static ref LOG: Regex = Regex::new(
        r"(?P<log_truncated>^Log truncated$)|(?P<program_invoke>^Program (?P<invoke_program_id>[1-9A-HJ-NP-Za-km-z]{32,}) invoke \[(?P<level>\d+)\]$)|(?P<program_success_result>^Program (?P<success_result_program_id>[1-9A-HJ-NP-Za-km-z]{32,}) success$)|(?P<program_failed_result>^Program (?P<failed_result_program_id>[1-9A-HJ-NP-Za-km-z]{32,}) failed: (?P<failed_result_err>.*)$)|(?P<program_complete_failed_result>^Program failed to complete: (?P<failed_complete_error>.*)$)|(?P<program_log>^^Program log: (?P<log_message>(.*[\n]?)+))|(?P<program_data>^Program data: (?P<data>(.*[\n]?)+))|(?P<program_consumed>^Program (?P<consumed_program_id>[1-9A-HJ-NP-Za-km-z]{32,}) consumed (?P<consumed_compute_units>\d*) of (?P<all_computed_units>\d*) compute units$)|(?P<program_return>^Program return: (?P<return_program_id>[1-9A-HJ-NP-Za-km-z]{32,}) (?P<return_message>(.*[\n]?)+))"
    )
    .expect("Failed to compile log regexp");
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum Error {
    #[error(transparent)]
    Base58Error(#[from] bs58::decode::Error),
    #[error(transparent)]
    ParseLevelError(#[from] std::num::ParseIntError),
    #[error("Wrong pubkey size: {0}")]
    WrongPubkeySize(String),
    #[error("Bind event error")]
    BindEventError,
    #[error("Bad log line: {0}")]
    BadLogLine(String),
    #[error("Unexpected program result at index {index}. Program {program_id:?} level: {level:?}")]
    UnexpectedProgramResult {
        index: usize,
        program_id: Pubkey,
        expected_program: Option<Pubkey>,
        level: Option<Level>,
    },
    #[error("Error log via binding logs: {program_id:?}, error: {err} at index {index}")]
    ErrorLog {
        program_id: Pubkey,
        err: String,
        index: usize,
    },
    #[error("Error to complete log via binding logs: error: {err} at index {index}")]
    ErrorToCompleteLog { err: String, index: usize },
    #[error("Missplace consumed log: Invoked {expected_program:?}, consumed program id: {consumed_program_id:?} at index {index}")]
    MissplaceConsumed {
        consumed_program_id: Pubkey,
        expected_program: Option<Pubkey>,
        index: usize,
    },
    #[error("Missing invoke log context {index}")]
    EmptyInvokeLogContext { index: usize },
    #[error("Log parser corrupted")]
    ErrorInRegexp,
}

#[cfg(feature = "solana")]
impl From<solana_sdk::pubkey::ParsePubkeyError> for Error {
    fn from(err: solana_sdk::pubkey::ParsePubkeyError) -> Self {
        Self::WrongPubkeySize(err.to_string())
    }
}

pub type Level = NonZeroU8;

#[derive(Debug, PartialEq, Eq)]
pub enum Log {
    Truncated,
    ProgramInvoke {
        program_id: Pubkey,
        level: Level,
    },
    ProgramResult {
        program_id: Pubkey,
        err: Option<String>,
    },
    ProgramFailedComplete {
        err: String,
    },
    ProgramLog {
        log: String,
    },
    ProgramData {
        data: String,
    },
    ProgramReturn {
        program_id: Pubkey,
        data: String,
    },
    ProgramConsumed {
        program_id: Pubkey,
        consumed: usize,
        all: usize,
    },
}

impl Log {
    fn new(input: &str) -> Result<Self, Error> {
        let capture = LOG
            .captures(input)
            .ok_or_else(|| Error::BadLogLine(input.to_string()))?;

        if capture.name("log_truncated").is_some() {
            Ok(Log::Truncated)
        } else if capture.name("program_invoke").is_some() {
            Ok(Log::ProgramInvoke {
                program_id: Pubkey::from_str(
                    capture
                        .name("invoke_program_id")
                        .ok_or(Error::ErrorInRegexp)?
                        .as_str(),
                )?,
                level: capture
                    .name("level")
                    .ok_or(Error::ErrorInRegexp)?
                    .as_str()
                    .parse()?,
            })
        } else if capture.name("program_success_result").is_some() {
            Ok(Log::ProgramResult {
                program_id: Pubkey::from_str(
                    capture
                        .name("success_result_program_id")
                        .ok_or(Error::ErrorInRegexp)?
                        .as_str(),
                )?,
                err: None,
            })
        } else if capture.name("program_failed_result").is_some() {
            Ok(Log::ProgramResult {
                program_id: Pubkey::from_str(
                    capture
                        .name("failed_result_program_id")
                        .ok_or(Error::ErrorInRegexp)?
                        .as_str(),
                )?,
                err: Some(
                    capture
                        .name("failed_result_err")
                        .ok_or(Error::ErrorInRegexp)?
                        .as_str()
                        .to_owned(),
                ),
            })
        } else if capture.name("program_complete_failed_result").is_some() {
            Ok(Log::ProgramFailedComplete {
                err: capture
                    .name("failed_complete_error")
                    .ok_or(Error::ErrorInRegexp)?
                    .as_str()
                    .to_owned(),
            })
        } else if capture.name("program_return").is_some() {
            Ok(Log::ProgramReturn {
                program_id: Pubkey::from_str(
                    capture
                        .name("return_program_id")
                        .ok_or(Error::ErrorInRegexp)?
                        .as_str(),
                )?,
                data: capture
                    .name("return_message")
                    .ok_or(Error::ErrorInRegexp)?
                    .as_str()
                    .to_owned(),
            })
        } else if capture.name("program_log").is_some() {
            Ok(Log::ProgramLog {
                log: capture
                    .name("log_message")
                    .ok_or(Error::ErrorInRegexp)?
                    .as_str()
                    .to_owned(),
            })
        } else if capture.name("program_data").is_some() {
            Ok(Log::ProgramData {
                data: capture
                    .name("data")
                    .ok_or(Error::ErrorInRegexp)?
                    .as_str()
                    .to_owned(),
            })
        } else if capture.name("program_consumed").is_some() {
            Ok(Log::ProgramConsumed {
                program_id: Pubkey::from_str(
                    capture
                        .name("consumed_program_id")
                        .ok_or(Error::ErrorInRegexp)?
                        .as_str(),
                )?,
                consumed: capture
                    .name("consumed_compute_units")
                    .ok_or(Error::ErrorInRegexp)?
                    .as_str()
                    .parse()?,
                all: capture
                    .name("all_computed_units")
                    .ok_or(Error::ErrorInRegexp)?
                    .as_str()
                    .parse()?,
            })
        } else {
            Err(Error::BadLogLine(input.to_owned()))
        }
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProgramLog {
    Data(String),
    Log(String),
    Return(ProgramReturn),
    Invoke(ProgramContext),
    Consumed { consumed: usize, all: usize },
}

#[derive(Clone, Hash, PartialEq, Eq, Debug, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ProgramReturn {
    pub program_id: Pubkey,
    pub data: String,
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ProgramContext {
    pub program_id: Pubkey,
    pub call_index: usize,
    pub invoke_level: NonZeroU8,
}

pub fn bind_events(
    input: impl Iterator<Item = Result<Log, Error>>,
) -> Result<HashMap<ProgramContext, Vec<ProgramLog>>, Error> {
    let mut programs_stack: Vec<ProgramContext> = vec![];
    let last_at_stack = |stack: &[ProgramContext], index: usize| {
        stack
            .last()
            .copied()
            .ok_or(Error::EmptyInvokeLogContext { index })
    };
    let mut call_index_map = HashMap::new();
    let mut get_and_update_call_index = move |program_id| {
        let i = call_index_map.entry(program_id).or_insert(0);
        let call_index = *i;
        *i += 1;
        call_index
    };

    let mut result = HashMap::<ProgramContext, Vec<ProgramLog>>::new();
    for (index, log) in input.enumerate() {
        match log? {
            Log::Truncated => {
                log::debug!("\"Log truncated\" found at index {}", index);
                break;
            }
            Log::ProgramInvoke { program_id, level } => {
                let new_ctx = ProgramContext {
                    program_id,
                    invoke_level: level,
                    call_index: get_and_update_call_index(program_id),
                };
                if let Ok(ctx) = last_at_stack(&programs_stack, index) {
                    result
                        .entry(ctx)
                        .or_default()
                        .push(ProgramLog::Invoke(new_ctx));
                }

                programs_stack.push(new_ctx);
                result
                    .entry(last_at_stack(&programs_stack, index)?)
                    .or_default();
            }
            Log::ProgramResult {
                program_id: finished_program_id,
                err: None,
            } => match programs_stack.pop() {
                Some(ctx) if ctx.program_id.eq(&finished_program_id) => {}
                Some(ctx) => {
                    return Err(Error::UnexpectedProgramResult {
                        index,
                        program_id: ctx.program_id,
                        level: Some(ctx.invoke_level),
                        expected_program: Some(finished_program_id),
                    });
                }
                None => {
                    return Err(Error::UnexpectedProgramResult {
                        index,
                        program_id: finished_program_id,
                        level: None,
                        expected_program: None,
                    });
                }
            },
            Log::ProgramResult {
                program_id,
                err: Some(err),
            } => {
                return Err(Error::ErrorLog {
                    program_id,
                    err,
                    index,
                });
            }
            Log::ProgramFailedComplete { err } => {
                return Err(Error::ErrorToCompleteLog { err, index });
            }
            Log::ProgramLog { log } => {
                result
                    .entry(last_at_stack(&programs_stack, index)?)
                    .or_default()
                    .push(ProgramLog::Log(log));
            }
            Log::ProgramReturn { program_id, data } => {
                result
                    .entry(last_at_stack(&programs_stack, index)?)
                    .or_default()
                    .push(ProgramLog::Return(ProgramReturn { program_id, data }));
            }
            Log::ProgramData { data } => result
                .entry(last_at_stack(&programs_stack, index)?)
                .or_default()
                .push(ProgramLog::Data(data)),
            Log::ProgramConsumed {
                program_id,
                consumed,
                all,
            } => {
                let ctx = last_at_stack(&programs_stack, index)?;
                if program_id.ne(&ctx.program_id) {
                    return Err(Error::MissplaceConsumed {
                        expected_program: Some(ctx.program_id),
                        consumed_program_id: program_id,
                        index,
                    });
                }
                result
                    .entry(last_at_stack(&programs_stack, index)?)
                    .or_default()
                    .push(ProgramLog::Consumed { consumed, all });
                log::info!(
                    "Program {:?} at level {}, consumed {}, all: {}",
                    bs58::encode(&ctx.program_id).into_string(),
                    ctx.invoke_level,
                    consumed,
                    all
                );
            }
        };
    }

    Ok(result)
}

pub fn parse_events(input: &[String]) -> Result<HashMap<ProgramContext, Vec<ProgramLog>>, Error> {
    bind_events(input.iter().map(|input_log| Log::new(input_log)))
}

#[cfg(test)]
mod log_test {
    use std::{collections::BTreeMap, str::FromStr};

    use super::*;
    #[test]
    fn test_truncated() {
        assert_eq!(
            Log::new("Log truncated").expect("Failed to check log"),
            Log::Truncated
        );
    }
    #[test]
    fn test_invoke() {
        assert_eq!(
            Log::new("Program M2mx93ekt1fmXSVkTrUL9xVFHkmME8HTUi5Cyc5aF7K invoke [1]")
                .expect("Failed to check log"),
            Log::ProgramInvoke {
                program_id: Pubkey::from_str("M2mx93ekt1fmXSVkTrUL9xVFHkmME8HTUi5Cyc5aF7K")
                    .unwrap(),
                level: Level::new(1).unwrap(),
            }
        );
    }
    #[test]
    fn test_result() {
        assert_eq!(
            Log::new("Program M2mx93ekt1fmXSVkTrUL9xVFHkmME8HTUi5Cyc5aF7K success")
                .expect("Failed to check log"),
            Log::ProgramResult {
                program_id: Pubkey::from_str("M2mx93ekt1fmXSVkTrUL9xVFHkmME8HTUi5Cyc5aF7K")
                    .unwrap(),
                err: None
            }
        );
    }
    #[test]
    fn test_log() {
        assert_eq!(
            Log::new("Program log: Instruction Deposit").expect("Failed to check log"),
            Log::ProgramLog {
                log: "Instruction Deposit".to_owned(),
            }
        );
    }
    #[test]
    fn test_return() {
        assert_eq!(
            Log::new("Program return: M2mx93ekt1fmXSVkTrUL9xVFHkmME8HTUi5Cyc5aF7K Some return")
                .expect("Failed to check log"),
            Log::ProgramReturn {
                program_id: Pubkey::from_str("M2mx93ekt1fmXSVkTrUL9xVFHkmME8HTUi5Cyc5aF7K")
                    .unwrap(),
                data: "Some return".to_owned(),
            }
        );
    }
    #[test]
    fn test_return_multiline() {
        assert_eq!(
            Log::new(
                "Program return: M2mx93ekt1fmXSVkTrUL9xVFHkmME8HTUi5Cyc5aF7K Some return
            newline return
            more newline return"
            )
            .expect("Failed to check log"),
            Log::ProgramReturn {
                program_id: Pubkey::from_str("M2mx93ekt1fmXSVkTrUL9xVFHkmME8HTUi5Cyc5aF7K")
                    .unwrap(),
                data: "Some return
            newline return
            more newline return"
                    .to_owned(),
            }
        );
    }
    #[test]
    fn test_log_multiline() {
        assert_eq!(
            Log::new(
                "Program log: Instruction Deposit
            Yet another instruction Deposit 1
            Yet another instruction Deposit 2
            Yet another instruction Deposit 3"
            )
            .expect("Failed to check log"),
            Log::ProgramLog {
                log: "Instruction Deposit
            Yet another instruction Deposit 1
            Yet another instruction Deposit 2
            Yet another instruction Deposit 3"
                    .to_owned(),
            }
        );
    }
    #[test]
    fn test_data_multiline() {
        assert_eq!(
            Log::new(
                "Program data: DATADATADATA
            MOREDATA
            SOMEMOREDATA"
            )
            .expect("Failed to check log"),
            Log::ProgramData {
                data: "DATADATADATA
            MOREDATA
            SOMEMOREDATA"
                    .to_owned(),
            }
        );
    }
    #[test]
    fn test_data() {
        assert_eq!(
            Log::new("Program data: DATADATADATA").expect("Failed to check log"),
            Log::ProgramData {
                data: "DATADATADATA".to_owned(),
            }
        );
    }
    #[test]
    fn test_consumed() {
        assert_eq!(
            Log::new("Program M2mx93ekt1fmXSVkTrUL9xVFHkmME8HTUi5Cyc5aF7K consumed 9297 of 1400000 compute units").expect("Failed to check log"),
            Log::ProgramConsumed {
                program_id: Pubkey::from_str("M2mx93ekt1fmXSVkTrUL9xVFHkmME8HTUi5Cyc5aF7K").unwrap(),
                consumed: 9297,
                all: 1400000,
            }
        );
    }

    const INPUT: &str = r##"Program M2mx93ekt1fmXSVkTrUL9xVFHkmME8HTUi5Cyc5aF7K invoke [1]
Program log: Instruction: Deposit
Program 11111111111111111111111111111111 invoke [2]
Program 11111111111111111111111111111111 success
Program M2mx93ekt1fmXSVkTrUL9xVFHkmME8HTUi5Cyc5aF7K consumed 9297 of 1400000 compute units
Program M2mx93ekt1fmXSVkTrUL9xVFHkmME8HTUi5Cyc5aF7K success
Program M2mx93ekt1fmXSVkTrUL9xVFHkmME8HTUi5Cyc5aF7K invoke [1]
Program log: Instruction: Buy
Program 11111111111111111111111111111111 invoke [2]
Program 11111111111111111111111111111111 success
Program log: {"price":17800000000,"buyer_expiry":0}
Program M2mx93ekt1fmXSVkTrUL9xVFHkmME8HTUi5Cyc5aF7K consumed 24562 of 1390703 compute units
Program M2mx93ekt1fmXSVkTrUL9xVFHkmME8HTUi5Cyc5aF7K success
Program M2mx93ekt1fmXSVkTrUL9xVFHkmME8HTUi5Cyc5aF7K invoke [1]
Program log: Instruction: ExecuteSale
Program 11111111111111111111111111111111 invoke [2]
Program 11111111111111111111111111111111 success
Program 11111111111111111111111111111111 invoke [2]
Program 11111111111111111111111111111111 success
Program 11111111111111111111111111111111 invoke [2]
Program 11111111111111111111111111111111 success
Program 11111111111111111111111111111111 invoke [2]
Program 11111111111111111111111111111111 success
Program ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL invoke [2]
Program log: Create
Program 11111111111111111111111111111111 invoke [3]
Program 11111111111111111111111111111111 success
Program log: Initialize the associated token account
Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [3]
Program log: Instruction: InitializeAccount3
Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 2629 of 1270540 compute units
Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success
Program ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL consumed 15295 of 1282583 compute units
Program ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL success
Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [2]
Program log: Instruction: Transfer
Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 2643 of 1261674 compute units
Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success
Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [2]
Program log: Instruction: CloseAccount
Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 1839 of 1244530 compute units
Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success
Program log: {"price":17800000000,"seller_expiry":-1,"buyer_expiry":0}
Program M2mx93ekt1fmXSVkTrUL9xVFHkmME8HTUi5Cyc5aF7K consumed 126365 of 1366141 compute units
Program M2mx93ekt1fmXSVkTrUL9xVFHkmME8HTUi5Cyc5aF7K success
Program JUP2jxvXaqu7NQY1GmNF4m1vodw12LVXYxbFL2uJvfo invoke [1]
Program log: Instruction: SetTokenLedger
Program JUP2jxvXaqu7NQY1GmNF4m1vodw12LVXYxbFL2uJvfo consumed 4314 of 1400000 compute units
Program JUP2jxvXaqu7NQY1GmNF4m1vodw12LVXYxbFL2uJvfo success
Program JUP2jxvXaqu7NQY1GmNF4m1vodw12LVXYxbFL2uJvfo invoke [1]
Program log: Instruction: SaberSwap
Program SSwpkEEcbUqx4vtoEByFjSkhKdCT862DNVb52nZg1UZ invoke [2]
Program log: Instruction: Swap
Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [3]
Program log: Instruction: Transfer
Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 2755 of 1336150 compute units
Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success
Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [3]
Program log: Instruction: Transfer
Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 2643 of 1331046 compute units
Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success
Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [3]
Program log: Instruction: Transfer
Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 2703 of 1326055 compute units
Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success
Program log: Event: SwapBToA
Program log: 0x3, 0x135f20cb4, 0x1432fe688, 0x0, 0xcb20
Program SSwpkEEcbUqx4vtoEByFjSkhKdCT862DNVb52nZg1UZ consumed 58110 of 1380870 compute units
Program SSwpkEEcbUqx4vtoEByFjSkhKdCT862DNVb52nZg1UZ success
Program JUP2jxvXaqu7NQY1GmNF4m1vodw12LVXYxbFL2uJvfo consumed 74537 of 1395686 compute units
Program JUP2jxvXaqu7NQY1GmNF4m1vodw12LVXYxbFL2uJvfo success
Program JUP2jxvXaqu7NQY1GmNF4m1vodw12LVXYxbFL2uJvfo invoke [1]
Program log: Instruction: MercurialExchange
Program MERLuDFBMmsHnsBPZw2sDQZHvXFMwp8EdjudcU2HKky invoke [2]
Program log: Instruction: Exchange
Program log: Total iteration: 4
Program log: GetFeeAmount: {"amount": 54220, "index": 1}
Program log: GetDyUnderlying: {"dy": 5422028523}
Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [3]
Program log: Instruction: Transfer
Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 2643 of 1237818 compute units
Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success
Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [3]
Program log: Instruction: Transfer
Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 2755 of 1232865 compute units
Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success
Program MERLuDFBMmsHnsBPZw2sDQZHvXFMwp8EdjudcU2HKky consumed 72081 of 1301637 compute units
Program MERLuDFBMmsHnsBPZw2sDQZHvXFMwp8EdjudcU2HKky success
Program log: AnchorError occurred. Error Code: SlippageToleranceExceeded. Error Number: 6000. Error Message: Slippage tolerance exceeded.
Program JUP2jxvXaqu7NQY1GmNF4m1vodw12LVXYxbFL2uJvfo consumed 96225 of 1321149 compute units
Program JUP2jxvXaqu7NQY1GmNF4m1vodw12LVXYxbFL2uJvfo failed: custom program error: 0x1770
Program JUP2jxvXaqu7NQY1GmNF4m1vodw12LVXYxbFL2uJvfo failed: custom program error: 0x1770
Program BRTbgHnC2AWfumCBU6ExthDie912RiDyiS3uXgMPQPQN invoke [1]
Program log: Instruction: ExecuteProposal
Program 11111111111111111111111111111111 invoke [2]
Program 11111111111111111111111111111111 success
Program BRTbgHnC2AWfumCBU6ExthDie912RiDyiS3uXgMPQPQN invoke [2]
Program log: Instruction: AddMember
Program 11111111111111111111111111111111 invoke [3]
Program 11111111111111111111111111111111 success
Program BRTbgHnC2AWfumCBU6ExthDie912RiDyiS3uXgMPQPQN consumed 170835 of 170835 compute units
Program failed to complete: exceeded maximum number of instructions allowed (170835) at instruction #40861
Program BRTbgHnC2AWfumCBU6ExthDie912RiDyiS3uXgMPQPQN failed: Program failed to complete
Program BRTbgHnC2AWfumCBU6ExthDie912RiDyiS3uXgMPQPQN consumed 200000 of 200000 compute units
Program BRTbgHnC2AWfumCBU6ExthDie912RiDyiS3uXgMPQPQN failed: Program failed to complete
Program return: BRTbgHnC2AWfumCBU6ExthDie912RiDyiS3uXgMPQ123 some return
Log truncated"##;
    #[test]
    fn test_parse() {
        let errors = INPUT
            .split('\n')
            .filter_map(|line| Some((line, Log::new(line).err()?)))
            .collect::<Vec<_>>();
        assert_eq!(errors, vec![]);
        super::parse_events(
            &INPUT
                .split('\n')
                .map(|s| s.to_owned())
                .take(89)
                .collect::<Vec<_>>(),
        )
        .unwrap();
        let program = r##"Program M2mx93ekt1fmXSVkTrUL9xVFHkmME8HTUi5Cyc5aF7K invoke [1]
Program log: Instruction: Deposit
Program 11111111111111111111111111111111 invoke [2]
Program 11111111111111111111111111111111 success
Program M2mx93ekt1fmXSVkTrUL9xVFHkmME8HTUi5Cyc5aF7K consumed 9297 of 1400000 compute units
Program M2mx93ekt1fmXSVkTrUL9xVFHkmME8HTUi5Cyc5aF7K success
Program M2mx93ekt1fmXSVkTrUL9xVFHkmME8HTUi5Cyc5aF7K invoke [1]
Program log: Instruction: Buy
Program 11111111111111111111111111111111 invoke [2]
Program 11111111111111111111111111111111 success
Program log: {"price":17800000000,"buyer_expiry":0}
Program M2mx93ekt1fmXSVkTrUL9xVFHkmME8HTUi5Cyc5aF7K consumed 24562 of 1390703 compute units
Program M2mx93ekt1fmXSVkTrUL9xVFHkmME8HTUi5Cyc5aF7K success"##;
        let program_events = super::parse_events(
            &program
                .split('\n')
                .map(|s| s.to_owned())
                .collect::<Vec<_>>(),
        )
        .unwrap()
        .into_iter()
        .collect::<BTreeMap<_, _>>();
        let expected = [
            (
                ProgramContext {
                    program_id: Pubkey::from_str("M2mx93ekt1fmXSVkTrUL9xVFHkmME8HTUi5Cyc5aF7K")
                        .unwrap(),
                    call_index: 0,
                    invoke_level: Level::new(1).unwrap(),
                },
                vec![
                    ProgramLog::Log("Instruction: Deposit".to_owned()),
                    ProgramLog::Invoke(ProgramContext {
                        program_id: Pubkey::from_str("11111111111111111111111111111111").unwrap(),
                        call_index: 0,
                        invoke_level: Level::new(2).unwrap(),
                    }),
                    ProgramLog::Consumed {
                        consumed: 9297,
                        all: 1400000,
                    },
                ],
            ),
            (
                ProgramContext {
                    program_id: Pubkey::from_str("11111111111111111111111111111111").unwrap(),
                    call_index: 0,
                    invoke_level: Level::new(2).unwrap(),
                },
                vec![],
            ),
            (
                ProgramContext {
                    program_id: Pubkey::from_str("M2mx93ekt1fmXSVkTrUL9xVFHkmME8HTUi5Cyc5aF7K")
                        .unwrap(),
                    call_index: 1,
                    invoke_level: Level::new(1).unwrap(),
                },
                vec![
                    ProgramLog::Log("Instruction: Buy".to_owned()),
                    ProgramLog::Invoke(ProgramContext {
                        program_id: Pubkey::from_str("11111111111111111111111111111111").unwrap(),
                        call_index: 1,
                        invoke_level: Level::new(2).unwrap(),
                    }),
                    ProgramLog::Log("{\"price\":17800000000,\"buyer_expiry\":0}".to_owned()),
                    ProgramLog::Consumed {
                        consumed: 24562,
                        all: 1390703,
                    },
                ],
            ),
            (
                ProgramContext {
                    program_id: Pubkey::from_str("11111111111111111111111111111111").unwrap(),
                    call_index: 1,
                    invoke_level: Level::new(2).unwrap(),
                },
                vec![],
            ),
        ]
        .into_iter()
        .collect::<BTreeMap<_, _>>();

        assert_eq!(expected, program_events);

        let program = r##"Program M2mx93ekt1fmXSVkTrUL9xVFHkmME8HTUi5Cyc5aF7K invoke [1]
Program return: M2mx93ekt1fmXSVkTrUL9xVFHkmME8HTUi5Cyc5aF7K Some return
Program M2mx93ekt1fmXSVkTrUL9xVFHkmME8HTUi5Cyc5aF7K consumed 9297 of 1400000 compute units
Program M2mx93ekt1fmXSVkTrUL9xVFHkmME8HTUi5Cyc5aF7K success"##;
        let program_events = super::parse_events(
            &program
                .split('\n')
                .map(|s| s.to_owned())
                .collect::<Vec<_>>(),
        )
        .unwrap()
        .into_iter()
        .collect::<BTreeMap<_, _>>();
        let expected = [(
            ProgramContext {
                program_id: Pubkey::from_str("M2mx93ekt1fmXSVkTrUL9xVFHkmME8HTUi5Cyc5aF7K")
                    .unwrap(),
                call_index: 0,
                invoke_level: Level::new(1).unwrap(),
            },
            vec![
                ProgramLog::Return(ProgramReturn {
                    program_id: Pubkey::from_str("M2mx93ekt1fmXSVkTrUL9xVFHkmME8HTUi5Cyc5aF7K")
                        .unwrap(),
                    data: "Some return".to_owned(),
                }),
                ProgramLog::Consumed {
                    consumed: 9297,
                    all: 1400000,
                },
            ],
        )]
        .into_iter()
        .collect::<BTreeMap<_, _>>();

        assert_eq!(expected, program_events);
    }
}

#[cfg(not(feature = "solana"))]
mod pubkey {
    use std::{ops::Deref, str::FromStr};

    use serde::{Deserialize, Serialize};

    #[derive(
        Serialize, Deserialize, Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash,
    )]
    pub struct Pubkey([u8; 32]);
    impl Deref for Pubkey {
        type Target = [u8; 32];
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl AsRef<[u8]> for Pubkey {
        fn as_ref(&self) -> &[u8] {
            self.deref()
        }
    }
    impl FromStr for Pubkey {
        type Err = bs58::decode::Error;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let mut pk = Self([0u8; 32]);
            if bs58::decode(s).into(&mut pk.0)?.eq(&32) {
                Ok(pk)
            } else {
                Err(Self::Err::BufferTooSmall)
            }
        }
    }
}
