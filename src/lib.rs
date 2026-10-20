//! Stargazer: a UCI chess engine.
//!
//! Modules so far:
//! - `bitboard`: sets of squares packed into 64-bit numbers (the bit tricks live here)
//! - `board`: pieces, squares and a full position, read from and written as FEN
//! - `attacks`: ready-made tables of where knights, kings and pawns attack
//! - `movegen`: listing the moves in a position
//! - `eval`: scoring a position (material and where the pieces stand)
//! - `search`: looking ahead to choose a move (alpha-beta, iterative deepening)
//! - `perft`: counting move sequences, to prove move generation correct
//! - `uci`: the text protocol GUIs and Lichess use to talk to the engine
//!
//! The architecture behind them is in `docs/DESIGN.md`.

pub mod attacks;
pub mod bitboard;
pub mod board;
pub mod eval;
pub mod movegen;
pub mod perft;
pub mod search;
pub mod uci;

pub const NAME: &str = "Stargazer";
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const AUTHOR: &str = "CharlieWA01";
