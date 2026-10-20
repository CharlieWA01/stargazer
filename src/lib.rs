//! Stargazer: a UCI chess engine.
//!
//! Modules so far:
//! - `bitboard`: sets of squares packed into 64-bit numbers (the bit tricks live here)
//! - `board`: pieces, squares and a full position, read from and written as FEN
//! - `uci`: the text protocol GUIs and Lichess use to talk to the engine
//!
//! The architecture behind them is in `docs/DESIGN.md`.

pub mod bitboard;
pub mod board;
pub mod uci;

pub const NAME: &str = "Stargazer";
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const AUTHOR: &str = "CharlieWA01";
