use crate::log_parser::*;

#[cfg(test)]
mod truncated_parse_test {
    use std::{collections::BTreeMap, str::FromStr};
    use super::*;
    #[test]
    fn truncated_parse_test() {
        let program = r##"Program 5AuQiksMerK4ZjF24KA5JNQAMsnZ5oGGyzyVpnohobqN invoke [1]
Program log: Instruction: Rebalance
Program log: event_q_x: 0
Program log: event_q_y: 0
Program log: event_q_bids: 0
Program log: event_q_asks: 0
Program log: 1/ Cancelling all orders...
Program Zo1ggzTUKMY5bYnDvT5mtVeZxzf2FaLTbKkmvGUhUQk invoke [2]
Program log: Instruction: CancelAllPerpOrders
Program ZDx8a8jBqGmJyxi1whFxxCo5vG6Q9t4hTzW2GSixMKK invoke [3]
Program log: Pruned 5 bids and 5 asks
Program ZDx8a8jBqGmJyxi1whFxxCo5vG6Q9t4hTzW2GSixMKK consumed 38713 of 1142547 compute units
Program ZDx8a8jBqGmJyxi1whFxxCo5vG6Q9t4hTzW2GSixMKK success
Program Zo1ggzTUKMY5bYnDvT5mtVeZxzf2FaLTbKkmvGUhUQk consumed 76047 of 1172405 compute units
Program Zo1ggzTUKMY5bYnDvT5mtVeZxzf2FaLTbKkmvGUhUQk success
Program log: 2/ Settle Funds...
Program Zo1ggzTUKMY5bYnDvT5mtVeZxzf2FaLTbKkmvGUhUQk invoke [2]
Program log: Instruction: SettleFunds
Program log: df 0
Program Zo1ggzTUKMY5bYnDvT5mtVeZxzf2FaLTbKkmvGUhUQk consumed 22081 of 1088951 compute units
Program Zo1ggzTUKMY5bYnDvT5mtVeZxzf2FaLTbKkmvGUhUQk success
Program log: 4/ levy crank fee...
Program 11111111111111111111111111111111 invoke [2]
Program 11111111111111111111111111111111 success
Program log: Rebalance setup end
Program 5AuQiksMerK4ZjF24KA5JNQAMsnZ5oGGyzyVpnohobqN consumed 137632 of 1200000 compute units
Program 5AuQiksMerK4ZjF24KA5JNQAMsnZ5oGGyzyVpnohobqN success
Program 5AuQiksMerK4ZjF24KA5JNQAMsnZ5oGGyzyVpnohobqN invoke [1]
Program log: Instruction: PlaceOrders
Program log: event_q_x: 0
Program log: event_q_y: 0
Program log: event_q_bids: 0
Program log: event_q_asks: 0
Program log: zamm_total_x: 138737069584299
Program log: zamm_total_y: 4542458779723
Program log: new price lot: 3275
Program log: new size lot: 2458
Program log: new price lot: 3272
Program log: new size lot: 2460
Program log: new price lot: 3277
Program log: new size lot: 4800
Program Zo1ggzTUKMY5bYnDvT5mtVeZxzf2FaLTbKkmvGUhUQk invoke [2]
Program ZDx8a8jBqGmJyxi1whFxxCo5vG6Q9t4hTzW2GSixMKK invoke [3]
Program log: ZoDexInstruction: Ask
Program log: DEBUG/RDC_POS/IS_LNG/true
Program log: zo-log
Program data: HeBoMJZwwn/UIUiwdP///9QhSLB0////gDYPnr4PAACAPyRXxA8AAAAAAAAAAAAAAAAAAAAAAAAACRW5BQAAAAtiMgA7AAAAAAAAAAAAAAAA
Program ZDx8a8jBqGmJyxi1whFxxCo5vG6Q9t4hTzW2GSixMKK consumed 29279 of 970880 compute units
Program ZDx8a8jBqGmJyxi1whFxxCo5vG6Q9t4hTzW2GSixMKK success
Program ZDx8a8jBqGmJyxi1whFxxCo5vG6Q9t4hTzW2GSixMKK invoke [3]
Program log: ZoDexInstruction: Bid
Program log: DEBUG/INCR_POS/IS_LNG/true
Program log: zo-log
Program data: HeBoMJZwwn9UJk6AdP///9QhSLB0////gDYPnr4PAACAPyRXxA8AAAAAAAAAAAAAADZGugUAAAAACRW5BQAAAAxiMgA7AAAAAAAAAAAAAAAA
Program ZDx8a8jBqGmJyxi1whFxxCo5vG6Q9t4hTzW2GSixMKK consumed 28439 of 930477 compute units
Program ZDx8a8jBqGmJyxi1whFxxCo5vG6Q9t4hTzW2GSixMKK success
Program ZDx8a8jBqGmJyxi1whFxxCo5vG6Q9t4hTzW2GSixMKK invoke [3]
Program log: ZoDexInstruction: Ask
Program log: DEBUG/RDC_POS/IS_LNG/true
Program log: zo-log
Program data: HeBoMJZwwn9UJk6AdP///9QhSLB0////gFYJcbMPAACAPyRXxA8AAAAAAAAAAAAAADZGugUAAAAA6RrmEAAAAA1iMgA7AAAAAAAAAAAAAAAA
Program ZDx8a8jBqGmJyxi1whFxxCo5vG6Q9t4hTzW2GSixMKK consumed 29257 of 890916 compute units
Program ZDx8a8jBqGmJyxi1whFxxCo5vG6Q9t4hTzW2GSixMKK success
Program log: getting mark price
Program Zo1ggzTUKMY5bYnDvT5mtVeZxzf2FaLTbKkmvGUhUQk consumed 168079 of 1021890 compute units
Program Zo1ggzTUKMY5bYnDvT5mtVeZxzf2FaLTbKkmvGUhUQk success
Program log: Place order end
Program 5AuQiksMerK4ZjF24KA5JNQAMsnZ5oGGyzyVpnohobqN consumed 211105 of 1062368 compute units
Program 5AuQiksMerK4ZjF24KA5JNQAMsnZ5oGGyzyVpnohobqN success
Program 5AuQiksMerK4ZjF24KA5JNQAMsnZ5oGGyzyVpnohobqN invoke [1]
Program log: Instruction: PlaceOrders
Program log: event_q_x: 0
Program log: event_q_y: 0
Program log: event_q_bids: 0
Program log: event_q_asks: 0
Program log: zamm_total_x: 138737069584299
Program log: zamm_total_y: 4542458779723
Program log: new price lot: 3270
Program log: new size lot: 4810
Program log: new price lot: 3282
Program log: new size lot: 9943
Program log: new price lot: 3265
Program log: new size lot: 9996
Program Zo1ggzTUKMY5bYnDvT5mtVeZxzf2FaLTbKkmvGUhUQk invoke [2]
Program ZDx8a8jBqGmJyxi1whFxxCo5vG6Q9t4hTzW2GSixMKK invoke [3]
Program log: ZoDexInstruction: Bid
Program log: DEBUG/INCR_POS/IS_LNG/true
Program log: zo-log
Program data: HeBoMJZwwn/kDo4idP///9QhSLB0////gFYJcbMPAACAPyRXxA8AAAAAAAAAAAAAAPdB7RAAAAAA6RrmEAAAAA5iMgA7AAAAAAAAAAAAAAAA
Program ZDx8a8jBqGmJyxi1whFxxCo5vG6Q9t4hTzW2GSixMKK consumed 28572 of 759681 compute units
Program ZDx8a8jBqGmJyxi1whFxxCo5vG6Q9t4hTzW2GSixMKK success
Program ZDx8a8jBqGmJyxi1whFxxCo5vG6Q9t4hTzW2GSixMKK invoke [3]
Program log: ZoDexInstruction: Ask
Program log: DEBUG/RDC_POS/IS_LNG/true
Program log: zo-log
Program data: HeBoMJZwwn/kDo4idP///9QhSLB0////APGLSpwPAACAPyRXxA8AAAAAAAAAAAAAAPdB7RAAAACATpgMKAAAAA9iMgA7AAAAAAAAAAAAAAAA
Program ZDx8a8jBqGmJyxi1whFxxCo5vG6Q9t4hTzW2GSixMKK consumed 29673 of 719987 compute units
Program ZDx8a8jBqGmJyxi1whFxxCo5vG6Q9t4hTzW2GSixMKK success
Program ZDx8a8jBqGmJyxi1whFxxCo5vG6Q9t4hTzW2GSixMKK invoke [3]
Program log: ZoDexInstruction: Bid
Program log: DEBUG/INCR_POS/IS_LNG/true
Program log: zo-log
Program data: HeBoMJZwwn80CgZgc////9QhSLB0////APGLSpwPAACAPyRXxA8AAAAAAAAAAAAAAIVWMygAAACATpgMKAAAABBiMgA7AAAAAAAAAAAAAAAA
Program ZDx8a8jBqGmJyxi1whFxxCo5vG6Q9t4hTzW2GSixMKK consumed 28607 of 679190 compute units
Program ZDx8a8jBqGmJyxi1whFxxCo5vG6Q9t4hTzW2GSixMKK success
Program log: getting mark price
Program Zo1ggzTUKMY5bYnDvT5mtVeZxzf2FaLTbKkmvGUhUQk consumed 167957 of 810692 compute units
Program Zo1ggzTUKMY5bYnDvT5mtVeZxzf2FaLTbKkmvGUhUQk success
Program log: Place order end
Program 5AuQiksMerK4ZjF24KA5JNQAMsnZ5oGGyzyVpnohobqN consumed 211076 of 851263 compute units
Program 5AuQiksMerK4ZjF24KA5JNQAMsnZ5oGGyzyVpnohobqN success
Program 5AuQiksMerK4ZjF24KA5JNQAMsnZ5oGGyzyVpnohobqN invoke [1]
Program log: Instruction: PlaceOrders
Program log: event_q_x: 0
Program log: event_q_y: 0
Program log: event_q_bids: 0
Program log: event_q_asks: 0
Program log: zamm_total_x: 138737069584299
Program log: zamm_total_y: 4542458779723
Program log: new price lot: 3284
Program log: new size lot: 4817
Program log: new price lot: 3263
Program log: new size lot: 4859
Program log: new price lot: 3285
Program log: new size lot: 2957
Program Zo1ggzTUKMY5bYnDvT5mtVeZxzf2FaLTbKkmvGUhUQk invoke [2]
Program ZDx8a8jBqGmJyxi1whFxxCo5vG6Q9t4hTzW2GSixMKK invoke [3]
Program log: ZoDexInstruction: Ask
Program log: DEBUG/RDC_POS/IS_LNG/true
Program log: zo-log
Program data: HeBoMJZwwn80CgZgc////9QhSLB0////gBJkE5EPAACAPyRXxA8AAAAAAAAAAAAAAIVWMygAAAAALcBDMwAAABFiMgA7AAAAAAAAAAAAAAAA
Program ZDx8a8jBqGmJyxi1whFxxCo5vG6Q9t4hTzW2GSixMKK consumed 29505 of 548566 compute units
Program ZDx8a8jBqGmJyxi1whFxxCo5vG6Q9t4hTzW2GSixMKK success
Program ZDx8a8jBqGmJyxi1whFxxCo5vG6Q9t4hTzW2GSixMKK invoke [3]
Program log: ZoDexInstruction: Bid
Program log: DEBUG/INCR_POS/IS_LNG/true
Program log: zo-log
Program data: HeBoMJZwwn9AW4UBc////9QhSLB0////gBJkE5EPAACAPyRXxA8AAAAAAAAAAAAAgBSHgzMAAAAALcBDMwAAABJiMgA7AAAAAAAAAAAAAAAA
Program ZDx8a8jBqGmJyxi1whFxxCo5vG6Q9t4hTzW2GSixMKK consumed 29244 of 507937 compute units
Program ZDx8a8jBqGmJyxi1whFxxCo5vG6Q9t4hTzW2GSixMKK success
Program ZDx8a8jBqGmJyxi1whFxxCo5vG6Q9t4hTzW2GSixMKK invoke [3]
Program log: ZoDexInstruction: Ask
Program log: DEBUG/RDC_POS/IS_LNG/true
Program log: zo-log
Program data: HeBoMJZwwn9AW4UBc////9QhSLB0////AK7hMIoPAACAPyRXxA8AAAAAAAAAAAAAgBSHgzMAAACAkUImOgAAABNiMgA7AAAAAAAAAAAAAAAA
Program ZDx8a8jBqGmJyxi1whFxxCo5vG6Q9t4hTzW2GSixMKK consumed 29708 of 467571 compute units
Program ZDx8a8jBqGmJyxi1whFxxCo5vG6Q9t4hTzW2GSixMKK success
Program log: getting mark price
Program Zo1ggzTUKMY5bYnDvT5mtVeZxzf2FaLTbKkmvGUhUQk consumed 169561 of 599576 compute units
Program Zo1ggzTUKMY5bYnDvT5mtVeZxzf2FaLTbKkmvGUhUQk success
Program log: Place order end
Program 5AuQiksMerK4ZjF24KA5JNQAMsnZ5oGGyzyVpnohobqN consumed 212720 of 640187 compute units
Program 5AuQiksMerK4ZjF24KA5JNQAMsnZ5oGGyzyVpnohobqN success
Program 5AuQiksMerK4ZjF24KA5JNQAMsnZ5oGGyzyVpnohobqN invoke [1]
Program log: Instruction: PlaceOrders
Program log: event_q_x: 0
Program log: event_q_y: 0
Program log: event_q_bids: 0
Program log: event_q_asks: 0
Program log: zamm_total_x: 138737069584299
Program log: zamm_total_y: 4542458779723
Program log: new price lot: 3262
Program log: new size lot: 2987
Program log: new price lot: 0
Program log: new size lot: 0
Program log: new price lot: 0
Program log: new size lot: 0
Program Zo1ggzTUKMY5bYnDvT5mtVeZxzf2FaLTbKkmvGUhUQk invoke [2]
Program ZDx8a8jBqGmJyxi1whFxxCo5vG6Q9t4hTzW2GSixMKK invoke [3]
Program log: ZoDexInstruction: Bid
Program log: DEBUG/INCR_POS/IS_LNG/true
Program log: zo-log
Program data: HeBoMJZwwn/Yz3HHcv///9QhSLB0////AK7hMIoPAACAPyRXxA8AAAAAAAAAAAAAABzrdzoAAACAkUImOgAAABRiMgA7AAAAAAAAAAAAAAAA
Program ZDx8a8jBqGmJyxi1whFxxCo5vG6Q9t4hTzW2GSixMKK consumed 29447 of 336641 compute units
Program ZDx8a8jBqGmJyxi1whFxxCo5vG6Q9t4hTzW2GSixMKK success
Program log: getting mark price
Program Zo1ggzTUKMY5bYnDvT5mtVeZxzf2FaLTbKkmvGUhUQk consumed 88156 of 387502 compute units
Program Zo1ggzTUKMY5bYnDvT5mtVeZxzf2FaLTbKkmvGUhUQk success
Program log: Place order end
Program 5AuQiksMerK4ZjF24KA5JNQAMsnZ5oGGyzyVpnohobqN consumed 130669 of 427467 compute units
Program 5AuQiksMerK4ZjF24KA5JNQAMsnZ5oGGyzyVpnohobqN success
Program 5AuQiksMerK4ZjF24KA5JNQAMsnZ5oGGyzyVpnohobqN invoke [1]
Program log: Instruction: PlaceOrders
Program log: event_q_x: 0
Program log: event_q_y: 0
Program log: event_q_bids: 0
Program log: event_q_asks: 0
Program log: zamm_total_x: 138737069584299
Program log: zamm_total_y: 4542458779723
Program log: new price lot: 0
Program log: new size lot: 0
Program log: new price lot: 0
Program log: new size lot: 0
Program log: new price lot: 0
Program log: new size lot: 0
Program Zo1ggzTUKMY5bYnDvT5mtVeZxzf2FaLTbKkmvGUhUQk invoke [2]
Program log: getting mark price
Program Zo1ggzTUKMY5bYnDvT5mtVeZxzf2FaLTbKkmvGUhUQk consumed 47303 of 257303 compute units
Program Zo1ggzTUKMY5bYnDvT5mtVeZxzf2FaLTbKkmvGUhUQk success
Program log: refunding crank fee...
Program log: Place order end
Log truncated
"##;
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
                    program_id: Pubkey::from_str("11111111111111111111111111111111").unwrap(),
                    call_index: 0,
                    invoke_level: Level::new(2).unwrap(),
                },
                vec![],
            ),
            (
                ProgramContext {
                    program_id: Pubkey::from_str("ZDx8a8jBqGmJyxi1whFxxCo5vG6Q9t4hTzW2GSixMKK")
                        .unwrap(),
                    call_index: 0,
                    invoke_level: Level::new(3).unwrap(),
                },
                vec![
                    ProgramLog::Log("Pruned 5 bids and 5 asks".to_owned()),
                    ProgramLog::Consumed {
                        consumed: 38713,
                        all: 1142547,
                    },
                ],
            ),
            (
                ProgramContext {
                    program_id: Pubkey::from_str("ZDx8a8jBqGmJyxi1whFxxCo5vG6Q9t4hTzW2GSixMKK").unwrap(),
                    call_index: 1,
                    invoke_level: Level::new(3).unwrap()
                },
                vec![
                    ProgramLog::Log("ZoDexInstruction: Ask".to_owned()),
                    ProgramLog::Log("DEBUG/RDC_POS/IS_LNG/true".to_owned()),
                    ProgramLog::Log("zo-log".to_owned()),
                    ProgramLog::Data("HeBoMJZwwn/UIUiwdP///9QhSLB0////gDYPnr4PAACAPyRXxA8AAAAAAAAAAAAAAAAAAAAAAAAACRW5BQAAAAtiMgA7AAAAAAAAAAAAAAAA".to_owned()),
                    ProgramLog::Consumed { consumed: 29279, all: 970880 }
                ],
            ),
            (
                ProgramContext {
                    program_id: Pubkey::from_str("ZDx8a8jBqGmJyxi1whFxxCo5vG6Q9t4hTzW2GSixMKK").unwrap(),
                    call_index: 2,
                    invoke_level: Level::new(3).unwrap()
                },
                vec![
                    ProgramLog::Log("ZoDexInstruction: Bid".to_owned()),
                    ProgramLog::Log("DEBUG/INCR_POS/IS_LNG/true".to_owned()),
                    ProgramLog::Log("zo-log".to_owned()),
                    ProgramLog::Data("HeBoMJZwwn9UJk6AdP///9QhSLB0////gDYPnr4PAACAPyRXxA8AAAAAAAAAAAAAADZGugUAAAAACRW5BQAAAAxiMgA7AAAAAAAAAAAAAAAA".to_owned()),
                    ProgramLog::Consumed { consumed: 28439, all: 930477 }
                ],
            ),
            (
                ProgramContext {
                    program_id: Pubkey::from_str("ZDx8a8jBqGmJyxi1whFxxCo5vG6Q9t4hTzW2GSixMKK").unwrap(),
                    call_index: 3,
                    invoke_level: Level::new(3).unwrap()
                },
                vec![
                    ProgramLog::Log("ZoDexInstruction: Ask".to_owned()),
                    ProgramLog::Log("DEBUG/RDC_POS/IS_LNG/true".to_owned()),
                    ProgramLog::Log("zo-log".to_owned()),
                    ProgramLog::Data("HeBoMJZwwn9UJk6AdP///9QhSLB0////gFYJcbMPAACAPyRXxA8AAAAAAAAAAAAAADZGugUAAAAA6RrmEAAAAA1iMgA7AAAAAAAAAAAAAAAA".to_owned()),
                    ProgramLog::Consumed { consumed: 29257, all: 890916 }
                ],
            ),
            (
                ProgramContext {
                    program_id: Pubkey::from_str("ZDx8a8jBqGmJyxi1whFxxCo5vG6Q9t4hTzW2GSixMKK").unwrap(),
                    call_index: 4,
                    invoke_level: Level::new(3).unwrap()
                },
                vec![
                    ProgramLog::Log("ZoDexInstruction: Bid".to_owned()),
                    ProgramLog::Log("DEBUG/INCR_POS/IS_LNG/true".to_owned()),
                    ProgramLog::Log("zo-log".to_owned()),
                    ProgramLog::Data("HeBoMJZwwn/kDo4idP///9QhSLB0////gFYJcbMPAACAPyRXxA8AAAAAAAAAAAAAAPdB7RAAAAAA6RrmEAAAAA5iMgA7AAAAAAAAAAAAAAAA".to_owned()),
                    ProgramLog::Consumed { consumed: 28572, all: 759681 }
                ],
            ),
            (
                ProgramContext {
                    program_id: Pubkey::from_str("ZDx8a8jBqGmJyxi1whFxxCo5vG6Q9t4hTzW2GSixMKK").unwrap(),
                    call_index: 5,
                    invoke_level: Level::new(3).unwrap()
                },
                vec![
                    ProgramLog::Log("ZoDexInstruction: Ask".to_owned()),
                    ProgramLog::Log("DEBUG/RDC_POS/IS_LNG/true".to_owned()),
                    ProgramLog::Log("zo-log".to_owned()),
                    ProgramLog::Data("HeBoMJZwwn/kDo4idP///9QhSLB0////APGLSpwPAACAPyRXxA8AAAAAAAAAAAAAAPdB7RAAAACATpgMKAAAAA9iMgA7AAAAAAAAAAAAAAAA".to_owned()),
                    ProgramLog::Consumed { consumed: 29673, all: 719987 }
                ],
            ),
            (
                ProgramContext {
                    program_id: Pubkey::from_str("ZDx8a8jBqGmJyxi1whFxxCo5vG6Q9t4hTzW2GSixMKK").unwrap(),
                    call_index: 6,
                    invoke_level: Level::new(3).unwrap()
                },
                vec![
                    ProgramLog::Log("ZoDexInstruction: Bid".to_owned()),
                    ProgramLog::Log("DEBUG/INCR_POS/IS_LNG/true".to_owned()),
                    ProgramLog::Log("zo-log".to_owned()),
                    ProgramLog::Data("HeBoMJZwwn80CgZgc////9QhSLB0////APGLSpwPAACAPyRXxA8AAAAAAAAAAAAAAIVWMygAAACATpgMKAAAABBiMgA7AAAAAAAAAAAAAAAA".to_owned()),
                    ProgramLog::Consumed { consumed: 28607, all: 679190 }
                ],
            ),
            (
                ProgramContext {
                    program_id: Pubkey::from_str("ZDx8a8jBqGmJyxi1whFxxCo5vG6Q9t4hTzW2GSixMKK").unwrap(),
                    call_index: 7,
                    invoke_level: Level::new(3).unwrap()
                },
                vec![
                    ProgramLog::Log("ZoDexInstruction: Ask".to_owned()),
                    ProgramLog::Log("DEBUG/RDC_POS/IS_LNG/true".to_owned()),
                    ProgramLog::Log("zo-log".to_owned()),
                    ProgramLog::Data("HeBoMJZwwn80CgZgc////9QhSLB0////gBJkE5EPAACAPyRXxA8AAAAAAAAAAAAAAIVWMygAAAAALcBDMwAAABFiMgA7AAAAAAAAAAAAAAAA".to_owned()),
                    ProgramLog::Consumed { consumed: 29505, all: 548566 }
                ],
            ),
            (
                ProgramContext {
                    program_id: Pubkey::from_str("ZDx8a8jBqGmJyxi1whFxxCo5vG6Q9t4hTzW2GSixMKK").unwrap(),
                    call_index: 8,
                    invoke_level: Level::new(3).unwrap()
                },
                vec![
                    ProgramLog::Log("ZoDexInstruction: Bid".to_owned()),
                    ProgramLog::Log("DEBUG/INCR_POS/IS_LNG/true".to_owned()),
                    ProgramLog::Log("zo-log".to_owned()),
                    ProgramLog::Data("HeBoMJZwwn9AW4UBc////9QhSLB0////gBJkE5EPAACAPyRXxA8AAAAAAAAAAAAAgBSHgzMAAAAALcBDMwAAABJiMgA7AAAAAAAAAAAAAAAA".to_owned()),
                    ProgramLog::Consumed { consumed: 29244, all: 507937 }
                ],
            ),
            (
                ProgramContext {
                    program_id: Pubkey::from_str("ZDx8a8jBqGmJyxi1whFxxCo5vG6Q9t4hTzW2GSixMKK").unwrap(),
                    call_index: 9,
                    invoke_level: Level::new(3).unwrap()
                },
                vec![
                    ProgramLog::Log("ZoDexInstruction: Ask".to_owned()),
                    ProgramLog::Log("DEBUG/RDC_POS/IS_LNG/true".to_owned()),
                    ProgramLog::Log("zo-log".to_owned()),
                    ProgramLog::Data("HeBoMJZwwn9AW4UBc////9QhSLB0////AK7hMIoPAACAPyRXxA8AAAAAAAAAAAAAgBSHgzMAAACAkUImOgAAABNiMgA7AAAAAAAAAAAAAAAA".to_owned()),
                    ProgramLog::Consumed { consumed: 29708, all: 467571 },
                ],
            ),
            (
                ProgramContext {
                    program_id: Pubkey::from_str("ZDx8a8jBqGmJyxi1whFxxCo5vG6Q9t4hTzW2GSixMKK").unwrap(),
                    call_index: 10,
                    invoke_level: Level::new(3).unwrap()
                },
                vec![
                    ProgramLog::Log("ZoDexInstruction: Bid".to_owned()),
                    ProgramLog::Log("DEBUG/INCR_POS/IS_LNG/true".to_owned()),
                    ProgramLog::Log("zo-log".to_owned()),
                    ProgramLog::Data("HeBoMJZwwn/Yz3HHcv///9QhSLB0////AK7hMIoPAACAPyRXxA8AAAAAAAAAAAAAABzrdzoAAACAkUImOgAAABRiMgA7AAAAAAAAAAAAAAAA".to_owned()),
                    ProgramLog::Consumed { consumed: 29447, all: 336641 },
                ],
            ),
            (
                ProgramContext {
                    program_id: Pubkey::from_str("Zo1ggzTUKMY5bYnDvT5mtVeZxzf2FaLTbKkmvGUhUQk").unwrap(),
                    call_index: 0,
                    invoke_level: Level::new(2).unwrap()
                },
                vec![
                    ProgramLog::Log("Instruction: CancelAllPerpOrders".to_owned()),
                    ProgramLog::Invoke(
                        ProgramContext {
                            program_id: Pubkey::from_str("ZDx8a8jBqGmJyxi1whFxxCo5vG6Q9t4hTzW2GSixMKK").unwrap(),
                            call_index: 0,
                            invoke_level: Level::new(3).unwrap(),
                        }),
                    ProgramLog::Consumed { consumed: 76047, all: 1172405 },
                ],
            ),
            (
                ProgramContext {
                    program_id: Pubkey::from_str("Zo1ggzTUKMY5bYnDvT5mtVeZxzf2FaLTbKkmvGUhUQk").unwrap(),
                    call_index: 1,
                    invoke_level: Level::new(2).unwrap()
                },
                vec![
                    ProgramLog::Log("Instruction: SettleFunds".to_owned()),
                    ProgramLog::Log("df 0".to_owned()),
                    ProgramLog::Consumed { consumed: 22081, all: 1088951 }
                ],
            ),
            (
                ProgramContext {
                    program_id: Pubkey::from_str("Zo1ggzTUKMY5bYnDvT5mtVeZxzf2FaLTbKkmvGUhUQk").unwrap(),
                    call_index: 2,
                    invoke_level: Level::new(2).unwrap()
                },
                vec![
                    ProgramLog::Invoke(
                        ProgramContext {
                            program_id: Pubkey::from_str("ZDx8a8jBqGmJyxi1whFxxCo5vG6Q9t4hTzW2GSixMKK").unwrap(),
                            call_index: 1,
                            invoke_level: Level::new(3).unwrap(),
                        }),
                    ProgramLog::Invoke(
                        ProgramContext {
                            program_id: Pubkey::from_str("ZDx8a8jBqGmJyxi1whFxxCo5vG6Q9t4hTzW2GSixMKK").unwrap(),
                            call_index: 2,
                            invoke_level: Level::new(3).unwrap(),
                        }),
                    ProgramLog::Invoke(
                        ProgramContext {
                            program_id: Pubkey::from_str("ZDx8a8jBqGmJyxi1whFxxCo5vG6Q9t4hTzW2GSixMKK").unwrap(),
                            call_index: 3,
                            invoke_level: Level::new(3).unwrap(),
                        }),
                    ProgramLog::Log("getting mark price".to_owned()),
                    ProgramLog::Consumed { consumed: 168079, all: 1021890 }
                ],
            ),
            (
                ProgramContext {
                    program_id: Pubkey::from_str("Zo1ggzTUKMY5bYnDvT5mtVeZxzf2FaLTbKkmvGUhUQk").unwrap(),
                    call_index: 3,
                    invoke_level: Level::new(2).unwrap()
                },
                vec![
                    ProgramLog::Invoke(
                        ProgramContext {
                            program_id: Pubkey::from_str("ZDx8a8jBqGmJyxi1whFxxCo5vG6Q9t4hTzW2GSixMKK").unwrap(),
                            call_index: 4,
                            invoke_level: Level::new(3).unwrap(),
                        }),
                    ProgramLog::Invoke(
                        ProgramContext {
                            program_id: Pubkey::from_str("ZDx8a8jBqGmJyxi1whFxxCo5vG6Q9t4hTzW2GSixMKK").unwrap(),
                            call_index: 5,
                            invoke_level: Level::new(3).unwrap(),
                        }),
                    ProgramLog::Invoke(
                        ProgramContext {
                            program_id: Pubkey::from_str("ZDx8a8jBqGmJyxi1whFxxCo5vG6Q9t4hTzW2GSixMKK").unwrap(),
                            call_index: 6,
                            invoke_level: Level::new(3).unwrap(),
                        }),
                    ProgramLog::Log("getting mark price".to_owned()),
                    ProgramLog::Consumed { consumed: 167957, all: 810692 }
                ],
            ),
            (
                ProgramContext {
                    program_id: Pubkey::from_str("Zo1ggzTUKMY5bYnDvT5mtVeZxzf2FaLTbKkmvGUhUQk").unwrap(),
                    call_index: 4,
                    invoke_level: Level::new(2).unwrap()
                },
                vec![
                    ProgramLog::Invoke(
                        ProgramContext {
                            program_id: Pubkey::from_str("ZDx8a8jBqGmJyxi1whFxxCo5vG6Q9t4hTzW2GSixMKK").unwrap(),
                            call_index: 7,
                            invoke_level: Level::new(3).unwrap(),
                        }),
                    ProgramLog::Invoke(
                        ProgramContext {
                            program_id: Pubkey::from_str("ZDx8a8jBqGmJyxi1whFxxCo5vG6Q9t4hTzW2GSixMKK").unwrap(),
                            call_index: 8,
                            invoke_level: Level::new(3).unwrap(),
                        }),
                    ProgramLog::Invoke(
                        ProgramContext {
                            program_id: Pubkey::from_str("ZDx8a8jBqGmJyxi1whFxxCo5vG6Q9t4hTzW2GSixMKK").unwrap(),
                            call_index: 9,
                            invoke_level: Level::new(3).unwrap(),
                        }),
                    ProgramLog::Log("getting mark price".to_owned()),
                    ProgramLog::Consumed { consumed: 169561, all: 599576 }
                ],
            ),
            (
                ProgramContext {
                    program_id: Pubkey::from_str("Zo1ggzTUKMY5bYnDvT5mtVeZxzf2FaLTbKkmvGUhUQk").unwrap(),
                    call_index: 5,
                    invoke_level: Level::new(2).unwrap()
                },
                vec![
                    ProgramLog::Invoke(
                        ProgramContext {
                            program_id: Pubkey::from_str("ZDx8a8jBqGmJyxi1whFxxCo5vG6Q9t4hTzW2GSixMKK").unwrap(),
                            call_index: 10,
                            invoke_level: Level::new(3).unwrap(),
                        }),
                    ProgramLog::Log("getting mark price".to_owned()),
                    ProgramLog::Consumed { consumed: 88156, all: 387502 }
                ],
            ),
            (
                ProgramContext {
                    program_id: Pubkey::from_str("Zo1ggzTUKMY5bYnDvT5mtVeZxzf2FaLTbKkmvGUhUQk").unwrap(),
                    call_index: 6,
                    invoke_level: Level::new(2).unwrap()
                },
                vec![
                    ProgramLog::Log("getting mark price".to_owned()),
                    ProgramLog::Consumed { consumed: 47303, all: 257303 }
                ],
            ),
            (
                ProgramContext {
                    program_id: Pubkey::from_str("5AuQiksMerK4ZjF24KA5JNQAMsnZ5oGGyzyVpnohobqN").unwrap(),
                    call_index: 0,
                    invoke_level: Level::new(1).unwrap()
                },
                vec![
                    ProgramLog::Log("Instruction: Rebalance".to_owned()),
                    ProgramLog::Log("event_q_x: 0".to_owned()),
                    ProgramLog::Log("event_q_y: 0".to_owned()),
                    ProgramLog::Log("event_q_bids: 0".to_owned()),
                    ProgramLog::Log("event_q_asks: 0".to_owned()),
                    ProgramLog::Log("1/ Cancelling all orders...".to_owned()),
                    ProgramLog::Invoke(
                        ProgramContext {
                            program_id: Pubkey::from_str("Zo1ggzTUKMY5bYnDvT5mtVeZxzf2FaLTbKkmvGUhUQk").unwrap(),
                            call_index: 0,
                            invoke_level: Level::new(2).unwrap()
                        }),
                    ProgramLog::Log("2/ Settle Funds...".to_owned()),
                    ProgramLog::Invoke(
                        ProgramContext {
                            program_id: Pubkey::from_str("Zo1ggzTUKMY5bYnDvT5mtVeZxzf2FaLTbKkmvGUhUQk").unwrap(),
                            call_index: 1,
                            invoke_level: Level::new(2).unwrap()
                        }),
                    ProgramLog::Log("4/ levy crank fee...".to_owned()),
                    ProgramLog::Invoke(
                        ProgramContext {
                            program_id: Pubkey::from_str("11111111111111111111111111111111").unwrap(),
                            call_index: 0,
                            invoke_level: Level::new(2).unwrap()
                        }),
                    ProgramLog::Log("Rebalance setup end".to_owned()),
                    ProgramLog::Consumed { consumed: 137632, all: 1200000 },
                ],
            ),
            (
                ProgramContext {
                    program_id: Pubkey::from_str("5AuQiksMerK4ZjF24KA5JNQAMsnZ5oGGyzyVpnohobqN").unwrap(),
                    call_index: 1,
                    invoke_level: Level::new(1).unwrap()
                },
                vec![
                    ProgramLog::Log("Instruction: PlaceOrders".to_owned()),
                    ProgramLog::Log("event_q_x: 0".to_owned()),
                    ProgramLog::Log("event_q_y: 0".to_owned()),
                    ProgramLog::Log("event_q_bids: 0".to_owned()),
                    ProgramLog::Log("event_q_asks: 0".to_owned()),
                    ProgramLog::Log("zamm_total_x: 138737069584299".to_owned()),
                    ProgramLog::Log("zamm_total_y: 4542458779723".to_owned()),
                    ProgramLog::Log("new price lot: 3275".to_owned()),
                    ProgramLog::Log("new size lot: 2458".to_owned()),
                    ProgramLog::Log("new price lot: 3272".to_owned()),
                    ProgramLog::Log("new size lot: 2460".to_owned()),
                    ProgramLog::Log("new price lot: 3277".to_owned()),
                    ProgramLog::Log("new size lot: 4800".to_owned()),
                    ProgramLog::Invoke(
                        ProgramContext {
                            program_id: Pubkey::from_str("Zo1ggzTUKMY5bYnDvT5mtVeZxzf2FaLTbKkmvGUhUQk").unwrap(),
                            call_index: 2,
                            invoke_level: Level::new(2).unwrap()
                        }),
                    ProgramLog::Log("Place order end".to_owned()),
                    ProgramLog::Consumed { consumed: 211105, all: 1062368 }
                ],
            ),
            (
                ProgramContext {
                    program_id: Pubkey::from_str("5AuQiksMerK4ZjF24KA5JNQAMsnZ5oGGyzyVpnohobqN").unwrap(),
                    call_index: 2,
                    invoke_level: Level::new(1).unwrap()
                },
                vec![
                    ProgramLog::Log("Instruction: PlaceOrders".to_owned()),
                    ProgramLog::Log("event_q_x: 0".to_owned()),
                    ProgramLog::Log("event_q_y: 0".to_owned()),
                    ProgramLog::Log("event_q_bids: 0".to_owned()),
                    ProgramLog::Log("event_q_asks: 0".to_owned()),
                    ProgramLog::Log("zamm_total_x: 138737069584299".to_owned()),
                    ProgramLog::Log("zamm_total_y: 4542458779723".to_owned()),
                    ProgramLog::Log("new price lot: 3270".to_owned()),
                    ProgramLog::Log("new size lot: 4810".to_owned()),
                    ProgramLog::Log("new price lot: 3282".to_owned()),
                    ProgramLog::Log("new size lot: 9943".to_owned()),
                    ProgramLog::Log("new price lot: 3265".to_owned()),
                    ProgramLog::Log("new size lot: 9996".to_owned()),
                    ProgramLog::Invoke(
                        ProgramContext {
                            program_id: Pubkey::from_str("Zo1ggzTUKMY5bYnDvT5mtVeZxzf2FaLTbKkmvGUhUQk").unwrap(),
                            call_index: 3,
                            invoke_level: Level::new(2).unwrap()
                        }),
                    ProgramLog::Log("Place order end".to_owned()),
                    ProgramLog::Consumed { consumed: 211076, all: 851263 }
                ],
            ),
            (
                ProgramContext {
                    program_id: Pubkey::from_str("5AuQiksMerK4ZjF24KA5JNQAMsnZ5oGGyzyVpnohobqN").unwrap(),
                    call_index: 3,
                    invoke_level: Level::new(1).unwrap()
                },
                vec![
                    ProgramLog::Log("Instruction: PlaceOrders".to_owned()),
                    ProgramLog::Log("event_q_x: 0".to_owned()),
                    ProgramLog::Log("event_q_y: 0".to_owned()),
                    ProgramLog::Log("event_q_bids: 0".to_owned()),
                    ProgramLog::Log("event_q_asks: 0".to_owned()),
                    ProgramLog::Log("zamm_total_x: 138737069584299".to_owned()),
                    ProgramLog::Log("zamm_total_y: 4542458779723".to_owned()),
                    ProgramLog::Log("new price lot: 3284".to_owned()),
                    ProgramLog::Log("new size lot: 4817".to_owned()),
                    ProgramLog::Log("new price lot: 3263".to_owned()),
                    ProgramLog::Log("new size lot: 4859".to_owned()),
                    ProgramLog::Log("new price lot: 3285".to_owned()),
                    ProgramLog::Log("new size lot: 2957".to_owned()),
                    ProgramLog::Invoke(
                        ProgramContext {
                            program_id: Pubkey::from_str("Zo1ggzTUKMY5bYnDvT5mtVeZxzf2FaLTbKkmvGUhUQk").unwrap(),
                            call_index: 4,
                            invoke_level: Level::new(2).unwrap()
                        }),
                    ProgramLog::Log("Place order end".to_owned()),
                    ProgramLog::Consumed { consumed: 212720, all: 640187 }
                ],
            ),
            (
                ProgramContext {
                    program_id: Pubkey::from_str("5AuQiksMerK4ZjF24KA5JNQAMsnZ5oGGyzyVpnohobqN").unwrap(),
                    call_index: 4,
                    invoke_level: Level::new(1).unwrap()
                },
                vec![
                    ProgramLog::Log("Instruction: PlaceOrders".to_owned()),
                    ProgramLog::Log("event_q_x: 0".to_owned()),
                    ProgramLog::Log("event_q_y: 0".to_owned()),
                    ProgramLog::Log("event_q_bids: 0".to_owned()),
                    ProgramLog::Log("event_q_asks: 0".to_owned()),
                    ProgramLog::Log("zamm_total_x: 138737069584299".to_owned()),
                    ProgramLog::Log("zamm_total_y: 4542458779723".to_owned()),
                    ProgramLog::Log("new price lot: 3262".to_owned()),
                    ProgramLog::Log("new size lot: 2987".to_owned()),
                    ProgramLog::Log("new price lot: 0".to_owned()),
                    ProgramLog::Log("new size lot: 0".to_owned()),
                    ProgramLog::Log("new price lot: 0".to_owned()),
                    ProgramLog::Log("new size lot: 0".to_owned()),
                    ProgramLog::Invoke(
                        ProgramContext {
                            program_id: Pubkey::from_str("Zo1ggzTUKMY5bYnDvT5mtVeZxzf2FaLTbKkmvGUhUQk").unwrap(),
                            call_index: 5,
                            invoke_level: Level::new(2).unwrap()
                        }),
                    ProgramLog::Log("Place order end".to_owned()),
                    ProgramLog::Consumed { consumed: 130669, all: 427467 }
                ],
            ),
            (
                ProgramContext {
                    program_id: Pubkey::from_str("5AuQiksMerK4ZjF24KA5JNQAMsnZ5oGGyzyVpnohobqN").unwrap(),
                    call_index: 5,
                    invoke_level: Level::new(1).unwrap()
                },
                vec![
                    ProgramLog::Log("Instruction: PlaceOrders".to_owned()),
                    ProgramLog::Log("event_q_x: 0".to_owned()),
                    ProgramLog::Log("event_q_y: 0".to_owned()),
                    ProgramLog::Log("event_q_bids: 0".to_owned()),
                    ProgramLog::Log("event_q_asks: 0".to_owned()),
                    ProgramLog::Log("zamm_total_x: 138737069584299".to_owned()),
                    ProgramLog::Log("zamm_total_y: 4542458779723".to_owned()),
                    ProgramLog::Log("new price lot: 0".to_owned()),
                    ProgramLog::Log("new size lot: 0".to_owned()),
                    ProgramLog::Log("new price lot: 0".to_owned()),
                    ProgramLog::Log("new size lot: 0".to_owned()),
                    ProgramLog::Log("new price lot: 0".to_owned()),
                    ProgramLog::Log("new size lot: 0".to_owned()),
                    ProgramLog::Invoke(
                        ProgramContext {
                            program_id: Pubkey::from_str("Zo1ggzTUKMY5bYnDvT5mtVeZxzf2FaLTbKkmvGUhUQk").unwrap(),
                            call_index: 6,
                            invoke_level: Level::new(2).unwrap()
                        }),
                    ProgramLog::Log("refunding crank fee...".to_owned()),
                    ProgramLog::Log("Place order end".to_owned())
                ],
            ),
        ]
            .into_iter()
            .collect::<BTreeMap<_, _>>();

        assert_eq!(expected, program_events);
    }
}
