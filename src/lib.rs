//! Stargazer: a UCI chess engine.
//!
//! The module layout here is a placeholder. The real structure (board,
//! move generation, search, evaluation) is decided in `docs/DESIGN.md`.

pub mod uci;

pub const NAME: &str = "Stargazer";
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const AUTHOR: &str = "CharlieWA01";
