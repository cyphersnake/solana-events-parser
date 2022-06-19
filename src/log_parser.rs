use std::{collections::HashMap, fmt::Debug, num::NonZeroU8};

use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;

lazy_static! {
    static ref LOG: Regex = Regex::new(
        r"(?P<program_invoke>^Program (?P<invoke_program_id>[1-9A-HJ-NP-Za-km-z]{32,}) invoke \[(?P<level>\d+)\]$)|(?P<program_success_result>^Program (?P<success_result_program_id>[1-9A-HJ-NP-Za-km-z]{32,}) success$)|(?P<program_failed_result>^Program (?P<failed_result_program_id>[1-9A-HJ-NP-Za-km-z]{32,}) failed: (?P<failed_result_err>.*)$)|(?P<program_complete_failed_result>^Program failed to complete: (?P<failed_complete_error>.*)$)|(?P<program_log>^^Program log: (?P<log_message>.*)$)|(?P<program_data>^Program data: (?P<data>.*)$)|(?P<program_consumed>^Program (?P<consumed_program_id>[1-9A-HJ-NP-Za-km-z]{32,}) consumed (?P<consumed_compute_units>\d*) of (?P<all_computed_units>\d*) compute units$)"
    )
    .expect("Failed to compile log regexp");
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum Error {
    #[error(transparent)]
    Base58Error(#[from] bs58::decode::Error),
    #[error(transparent)]
    ParseLevelError(#[from] std::num::ParseIntError),
    #[error("Wrong pubkey size: {0:?}")]
    WrongPubkeySize(Vec<u8>),
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
}

pub type Level = NonZeroU8;

#[derive(Debug, PartialEq, Eq)]
pub enum Log {
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

        if capture.name("program_invoke").is_some() {
            Ok(Log::ProgramInvoke {
                program_id: Pubkey::new_from_array(
                    <[u8; 32]>::try_from(
                        bs58::decode(
                            capture
                                .name("invoke_program_id")
                                .expect("Unreachable.")
                                .as_str(),
                        )
                        .into_vec()?,
                    )
                    .map_err(Error::WrongPubkeySize)?,
                ),
                level: capture
                    .name("level")
                    .expect("Unreachable.")
                    .as_str()
                    .parse()?,
            })
        } else if capture.name("program_success_result").is_some() {
            Ok(Log::ProgramResult {
                program_id: Pubkey::new_from_array(
                    <[u8; 32]>::try_from(
                        bs58::decode(
                            capture
                                .name("success_result_program_id")
                                .expect("Unreachable.")
                                .as_str(),
                        )
                        .into_vec()?,
                    )
                    .map_err(Error::WrongPubkeySize)?,
                ),
                err: None,
            })
        } else if capture.name("program_failed_result").is_some() {
            Ok(Log::ProgramResult {
                program_id: Pubkey::new_from_array(
                    <[u8; 32]>::try_from(
                        bs58::decode(
                            capture
                                .name("failed_result_program_id")
                                .expect("Unreachable.")
                                .as_str(),
                        )
                        .into_vec()?,
                    )
                    .map_err(Error::WrongPubkeySize)?,
                ),
                err: Some(
                    capture
                        .name("failed_result_err")
                        .expect("Unreachable.")
                        .as_str()
                        .to_owned(),
                ),
            })
        } else if capture.name("program_complete_failed_result").is_some() {
            Ok(Log::ProgramFailedComplete {
                err: capture
                    .name("failed_complete_error")
                    .expect("Unreachable.")
                    .as_str()
                    .to_owned(),
            })
        } else if capture.name("program_log").is_some() {
            Ok(Log::ProgramLog {
                log: capture
                    .name("log_message")
                    .expect("Unreachable.")
                    .as_str()
                    .to_owned(),
            })
        } else if capture.name("program_data").is_some() {
            Ok(Log::ProgramData {
                data: capture
                    .name("data")
                    .expect("Unreachable.")
                    .as_str()
                    .to_owned(),
            })
        } else if capture.name("program_consumed").is_some() {
            Ok(Log::ProgramConsumed {
                program_id: Pubkey::new_from_array(
                    <[u8; 32]>::try_from(
                        bs58::decode(
                            capture
                                .name("consumed_program_id")
                                .expect("Unreachable.")
                                .as_str(),
                        )
                        .into_vec()?,
                    )
                    .map_err(Error::WrongPubkeySize)?,
                ),
                consumed: capture
                    .name("consumed_compute_units")
                    .expect("Unreachable.")
                    .as_str()
                    .parse()?,
                all: capture
                    .name("all_computed_units")
                    .expect("Unreachable.")
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
    Invoke(ProgramContext),
    Consumed { consumed: usize, all: usize },
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

    input.enumerate().try_fold(
        HashMap::<ProgramContext, Vec<ProgramLog>>::new(),
        |mut result, (index, log)| {
            match log? {
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
                        bs58::encode(ctx.program_id).into_string(),
                        ctx.invoke_level,
                        consumed,
                        all
                    );
                }
            };
            Ok(result)
        },
    )
}

pub fn parse_events(input: &[String]) -> Result<HashMap<ProgramContext, Vec<ProgramLog>>, Error> {
    bind_events(input.iter().map(|input_log| Log::new(input_log)))
}

#[cfg(test)]
mod log_test {
    use std::collections::BTreeMap;

    use super::*;

    fn to_pubkey(input: &str) -> Result<Pubkey, Error> {
        Ok(Pubkey::new_from_array(
            <[u8; 32]>::try_from(bs58::decode(&input).into_vec()?)
                .map_err(Error::WrongPubkeySize)?,
        ))
    }

    #[test]
    fn test_invoke() {
        assert_eq!(
            Log::new("Program M2mx93ekt1fmXSVkTrUL9xVFHkmME8HTUi5Cyc5aF7K invoke [1]")
                .expect("Failed to check log"),
            Log::ProgramInvoke {
                program_id: to_pubkey("M2mx93ekt1fmXSVkTrUL9xVFHkmME8HTUi5Cyc5aF7K").unwrap(),
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
                program_id: to_pubkey("M2mx93ekt1fmXSVkTrUL9xVFHkmME8HTUi5Cyc5aF7K").unwrap(),
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
                program_id: to_pubkey("M2mx93ekt1fmXSVkTrUL9xVFHkmME8HTUi5Cyc5aF7K").unwrap(),
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
Program BRTbgHnC2AWfumCBU6ExthDie912RiDyiS3uXgMPQPQN failed: Program failed to complete"##;

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
                    program_id: to_pubkey("M2mx93ekt1fmXSVkTrUL9xVFHkmME8HTUi5Cyc5aF7K").unwrap(),
                    call_index: 0,
                    invoke_level: Level::new(1).unwrap(),
                },
                vec![
                    ProgramLog::Log("Instruction: Deposit".to_owned()),
                    ProgramLog::Invoke(ProgramContext {
                        program_id: to_pubkey("11111111111111111111111111111111").unwrap(),
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
                    program_id: to_pubkey("11111111111111111111111111111111").unwrap(),
                    call_index: 0,
                    invoke_level: Level::new(2).unwrap(),
                },
                vec![],
            ),
            (
                ProgramContext {
                    program_id: to_pubkey("M2mx93ekt1fmXSVkTrUL9xVFHkmME8HTUi5Cyc5aF7K").unwrap(),
                    call_index: 1,
                    invoke_level: Level::new(1).unwrap(),
                },
                vec![
                    ProgramLog::Log("Instruction: Buy".to_owned()),
                    ProgramLog::Invoke(ProgramContext {
                        program_id: to_pubkey("11111111111111111111111111111111").unwrap(),
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
                    program_id: to_pubkey("11111111111111111111111111111111").unwrap(),
                    call_index: 1,
                    invoke_level: Level::new(2).unwrap(),
                },
                vec![],
            ),
        ]
        .into_iter()
        .collect::<BTreeMap<_, _>>();

        assert_eq!(expected, program_events);
    }
}
