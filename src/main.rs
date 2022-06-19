pub mod log_parser {
    use std::{
        collections::HashMap,
        fmt::{Debug, Formatter, Result as FmtResult},
        num::NonZeroU8,
    };

    use diff::Diff;
    use lazy_static::lazy_static;
    use regex::Regex;

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
        #[error(
            "Unexpected program result at index {index}. Program {program_id:?} level: {level:?}"
        )]
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

    pub type Pubkey = [u8; 32];
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
                    program_id: <[u8; 32]>::try_from(
                        bs58::decode(
                            capture
                                .name("invoke_program_id")
                                .expect("Unreachable.")
                                .as_str(),
                        )
                        .into_vec()?,
                    )
                    .map_err(Error::WrongPubkeySize)?,
                    level: capture
                        .name("level")
                        .expect("Unreachable.")
                        .as_str()
                        .parse()?,
                })
            } else if capture.name("program_success_result").is_some() {
                Ok(Log::ProgramResult {
                    program_id: <[u8; 32]>::try_from(
                        bs58::decode(
                            capture
                                .name("success_result_program_id")
                                .expect("Unreachable.")
                                .as_str(),
                        )
                        .into_vec()?,
                    )
                    .map_err(Error::WrongPubkeySize)?,
                    err: None,
                })
            } else if capture.name("program_failed_result").is_some() {
                Ok(Log::ProgramResult {
                    program_id: <[u8; 32]>::try_from(
                        bs58::decode(
                            capture
                                .name("failed_result_program_id")
                                .expect("Unreachable.")
                                .as_str(),
                        )
                        .into_vec()?,
                    )
                    .map_err(Error::WrongPubkeySize)?,
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
                    program_id: <[u8; 32]>::try_from(
                        bs58::decode(
                            capture
                                .name("consumed_program_id")
                                .expect("Unreachable.")
                                .as_str(),
                        )
                        .into_vec()?,
                    )
                    .map_err(Error::WrongPubkeySize)?,
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

    #[derive(Debug, PartialEq, Eq, Diff)]
    #[diff(attr(
        #[derive(Debug)]
    ))]
    pub enum ProgramLog {
        Data(String),
        Log(String),
        Invoke(ProgramContext),
        Consumed { consumed: usize, all: usize },
    }

    #[derive(Clone, Copy, Hash, PartialEq, Eq, Diff)]
    #[diff(attr(
        #[derive(Debug)]
    ))]
    pub struct ProgramContext {
        pub program_id: Pubkey,
        pub call_index: usize,
        pub invoke_level: NonZeroU8,
    }
    impl Debug for ProgramContext {
        fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
            f.debug_struct("ProgramContext")
                .field("program_id", &bs58::encode(&self.program_id).into_string())
                .field("call_index", &self.call_index)
                .field("invoke_level", &self.call_index)
                .finish()
        }
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

    pub fn parse_events(
        input: &[String],
    ) -> Result<HashMap<ProgramContext, Vec<ProgramLog>>, Error> {
        bind_events(input.iter().map(|input_log| Log::new(input_log)))
    }

    #[cfg(test)]
    mod log_test {
        use super::*;

        fn to_pubkey(input: &str) -> Result<[u8; 32], Error> {
            <[u8; 32]>::try_from(bs58::decode(&input).into_vec()?).map_err(Error::WrongPubkeySize)
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
            .unwrap();
            let expected: HashMap<ProgramContext, Vec<ProgramLog>> = [
                (
                    ProgramContext {
                        program_id: to_pubkey("M2mx93ekt1fmXSVkTrUL9xVFHkmME8HTUi5Cyc5aF7K")
                            .unwrap(),
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
                        program_id: to_pubkey("M2mx93ekt1fmXSVkTrUL9xVFHkmME8HTUi5Cyc5aF7K")
                            .unwrap(),
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
            .collect();
            let diff = program_events.diff(&expected);
            assert!(
                diff.altered.is_empty() && diff.removed.is_empty(),
                "Altered: {:?}, Removed: {:?}",
                diff.altered,
                diff.removed
            );
        }
    }
}
pub mod transaction_parser {
    use std::{
        collections::HashMap,
        fmt::{Debug, Formatter, Result as FmtResult},
    };

    pub use solana_client::rpc_client::RpcClient;
    pub use solana_sdk::{
        instruction::{AccountMeta, Instruction},
        signature::Signature,
    };
    pub use solana_transaction_status::{
        EncodedTransactionWithStatusMeta, UiInstruction, UiTransactionEncoding,
    };

    pub use crate::log_parser::{self, ProgramContext, ProgramLog};

    #[derive(Debug, thiserror::Error)]
    pub enum Error {
        #[error(transparent)]
        SolanaClientResult(#[from] solana_client::client_error::ClientError),
        #[error(transparent)]
        LogParseError(#[from] crate::log_parser::Error),
        #[error("Field `meta` is empty in response of {0} tx request")]
        EmptyMetaInTransaction(Signature),
        #[error("Field `meta.log_messages` is empty in response of {0} tx request")]
        EmptyLogsInTransaction(Signature),
        #[error("Field `meta.inner_instructions` is empty")]
        EmptyInnerInstructionInTransaction(Signature),
        #[error("TODO")]
        ErrorWhileDecodeTransaction(Signature),
        #[error("TODO")]
        ErrorWhileDecodeData(bs58::decode::Error),
        #[error("TODO")]
        ParsedInnerInstructionNotSupported,
        #[error("Can't find ix ctx {0:?} in logs")]
        InstructionLogsConsistencyError(InstructionContext),
    }

    pub trait BindTransactionLogs {
        fn bind_transaction_logs(
            &self,
            signature: Signature,
        ) -> Result<HashMap<ProgramContext, Vec<ProgramLog>>, Error>;
    }
    impl BindTransactionLogs for RpcClient {
        fn bind_transaction_logs(
            &self,
            signature: Signature,
        ) -> Result<HashMap<ProgramContext, Vec<ProgramLog>>, Error> {
            Ok(log_parser::parse_events(
                self.get_transaction(&signature, UiTransactionEncoding::Base58)?
                    .transaction
                    .meta
                    .ok_or(Error::EmptyMetaInTransaction(signature))?
                    .log_messages
                    .ok_or(Error::EmptyLogsInTransaction(signature))?
                    .as_slice(),
            )?)
        }
    }

    #[derive(Clone, Copy, Hash, PartialEq, Eq)]
    pub struct InstructionContext {
        program_id: log_parser::Pubkey,
        call_index: usize,
    }
    impl Debug for InstructionContext {
        fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
            f.debug_struct("InstructionContext")
                .field("program_id", &bs58::encode(&self.program_id).into_string())
                .field("call_index", &self.call_index)
                .finish()
        }
    }

    pub type OuterInstruction = Option<log_parser::Pubkey>;

    pub trait BindInstructions {
        fn bind_instructions(
            &self,
            signature: Signature,
        ) -> Result<HashMap<InstructionContext, (Instruction, OuterInstruction)>, Error>;
    }
    impl BindInstructions for EncodedTransactionWithStatusMeta {
        fn bind_instructions(
            &self,
            signature: Signature,
        ) -> Result<HashMap<InstructionContext, (Instruction, OuterInstruction)>, Error> {
            let msg = self
                .transaction
                .decode()
                .ok_or(Error::ErrorWhileDecodeTransaction(signature))?
                .message;
            let accounts = msg.static_account_keys();

            let mut call_index_map = HashMap::new();
            let mut get_and_update_call_index = move |program_id| {
                let i = call_index_map.entry(program_id).or_insert(0);
                let call_index = *i;
                *i += 1;
                call_index
            };

            let inner_instructions = self
                .meta
                .as_ref()
                .ok_or(Error::EmptyMetaInTransaction(signature))?
                .inner_instructions
                .as_ref()
                .ok_or(Error::EmptyInnerInstructionInTransaction(signature))?
                .iter()
                .map(|ui_ix| (ui_ix.index as usize, &ui_ix.instructions))
                .collect::<HashMap<_, _>>();

            log::trace!(
                "Inner instructions: {:?} of {}",
                inner_instructions,
                signature
            );

            let mut result = HashMap::new();
            for (ix_index, compiled_ix) in msg.instructions().iter().enumerate() {
                log::trace!("Start handling instruction with index: {}", ix_index);

                let program_id = accounts[compiled_ix.program_id_index as usize];

                let ctx = InstructionContext {
                    program_id: program_id.to_bytes(),
                    call_index: get_and_update_call_index(program_id),
                };
                log::trace!("InstructionContext of {} ix is {:?}", ix_index, ctx);
                result.insert(
                    ctx,
                    (
                        Instruction {
                            program_id,
                            accounts: compiled_ix
                                .accounts
                                .iter()
                                .map(|&index| index as usize)
                                .map(|index| AccountMeta {
                                    pubkey: accounts[index],
                                    is_signer: msg.is_maybe_writable(index),
                                    is_writable: msg.is_signer(index),
                                })
                                .collect(),
                            data: compiled_ix.data.clone(),
                        },
                        None,
                    ),
                );
                if let Some(invokes) = inner_instructions.get(&ix_index) {
                    log::trace!(
                        "Found inner instruction {} for {} transaction instruction",
                        invokes.len(),
                        ix_index
                    );
                    for (invoke_index, invoke) in invokes.iter().enumerate() {
                        let invoke_ix = match invoke {
                            UiInstruction::Compiled(compiled) => Instruction {
                                program_id: accounts[compiled.program_id_index as usize],
                                accounts: compiled
                                    .accounts
                                    .iter()
                                    .map(|&index| index as usize)
                                    .map(|index| AccountMeta {
                                        pubkey: accounts[index],
                                        is_signer: msg.is_maybe_writable(index),
                                        is_writable: msg.is_signer(index),
                                    })
                                    .collect(),
                                data: bs58::decode(&compiled.data)
                                    .into_vec()
                                    .map_err(Error::ErrorWhileDecodeData)?,
                            },
                            UiInstruction::Parsed(_parsed) => {
                                return Err(Error::ParsedInnerInstructionNotSupported);
                            }
                        };
                        let ctx = InstructionContext {
                            program_id: invoke_ix.program_id.to_bytes(),
                            call_index: get_and_update_call_index(invoke_ix.program_id),
                        };
                        log::trace!(
                            "Invoke {} of ix {} with ctx {:?}",
                            invoke_index,
                            ix_index,
                            ctx
                        );
                        result.insert(ctx, (invoke_ix, Some(program_id.to_bytes())));
                    }
                }
            }

            Ok(result)
        }
    }

    pub trait BindTransactionInstructionLogs {
        fn bind_transaction_instruction_logs(
            &self,
            signature: Signature,
        ) -> Result<HashMap<ProgramContext, (Instruction, Vec<ProgramLog>)>, Error>;
    }
    impl BindTransactionInstructionLogs for RpcClient {
        fn bind_transaction_instruction_logs(
            &self,
            signature: Signature,
        ) -> Result<HashMap<ProgramContext, (Instruction, Vec<ProgramLog>)>, Error> {
            let tx = self
                .get_transaction(&signature, UiTransactionEncoding::Binary)?
                .transaction;
            let mut instructions = tx.bind_instructions(signature)?;

            log_parser::parse_events(
                tx.meta
                    .ok_or(Error::EmptyMetaInTransaction(signature))?
                    .log_messages
                    .ok_or(Error::EmptyLogsInTransaction(signature))?
                    .as_slice(),
            )?
            .into_iter()
            .map(|(ctx, events)| {
                let ix_ctx = InstructionContext {
                    program_id: ctx.program_id,
                    call_index: ctx.call_index,
                };
                let (ix, outer_ix) = instructions
                    .remove(&ix_ctx)
                    .ok_or(Error::InstructionLogsConsistencyError(ix_ctx))?;

                // TODO Add validation of outer ix
                if (outer_ix.is_none() && ctx.invoke_level.get() == 1)
                    || (outer_ix.is_some() && ctx.invoke_level.get() != 1)
                {
                    Ok((ctx, (ix, events)))
                } else {
                    Err(Error::InstructionLogsConsistencyError(ix_ctx))
                }
            })
            .collect()
        }
    }
}

use std::{env, str::FromStr};

use simple_logger::SimpleLogger;
use transaction_parser::*;

fn main() {
    SimpleLogger::new().env().init().unwrap();
    let client = RpcClient::new("https://api.mainnet-beta.solana.com");
    let events = client
        .bind_transaction_instruction_logs(
            Signature::from_str(&env::args().skip(1).next().unwrap()).unwrap(),
        )
        .unwrap();

    println!("{:?}", events);
}
