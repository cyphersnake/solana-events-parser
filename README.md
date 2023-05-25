# Solana Events Parser

[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](https://opensource.org/licenses/MIT)
[![Build Status](https://github.com/cyphersnake/solana-events-parser/actions/workflows/rust.yml/badge.svg)](https://github.com/cyphersnake/solana-events-parser/actions)

This is a Rust crate that provides various utilities for working with Solana blockchain transactions and events. The features of this crate include:

- Binding instructions from transaction into
```rust
pub struct TransactionParsedMeta {
    /// All internal instructions with logs
    pub meta: HashMap<ProgramContext, (Instruction, Vec<ProgramLog>)>,
    pub slot: Slot,
    pub block_time: Option<UnixTimestamp>,
    pub lamports_changes: HashMap<Pubkey, AmountDiff>,
    pub token_balances_changes: HashMap<WalletContext, AmountDiff>,
    pub parent_ix: HashMap<ChildProgramContext, ParentProgramContext>,
}
```
- Parsing logs of Solana programs
```rust
pub enum Log {
    DeployedProgram {
        program_id: Pubkey,
    },
    UpgradedProgram {
        program_id: Pubkey,
    },
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
    #[cfg(feature = "unknown_log")]
    UnknownFormat {
        unknown_log_string: String,
    },
}
```
- Parsing anchor based events into rust structure
- Automatic interception and processing of specific pubkey transactions using `event_reader_service`

## Installation

To use this crate in your project, add the following to your `Cargo.toml` file:

```toml
[dependencies.solana-events-parser]
version = "0.3.4"
git = "ssh://git@github.com/debridge-finance/solana-events-parser.git"
tag = "v0.3.4"
```

## Usage

To use this crate in your code, import the relevant modules using:

```rust
use solana_events_parser::*;
```

For more information on how to use this crate, refer to the documentation.

## License

This crate is distributed under the MIT license. For more information, see the LICENSE file.

