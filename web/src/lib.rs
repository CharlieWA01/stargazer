//! Stargazer in a web page: the bridge between the engine and JavaScript.
//!
//! The page loads this package compiled to WebAssembly (`stargazer.wasm`)
//! in a background worker, so the board stays responsive while the engine
//! thinks. There's no glue library (Charlie's choice, 2026-11-07): the
//! page and the engine trade text through WebAssembly's shared memory.
//!
//! - `game`: a game in progress (legal moves, the end of the game, move
//!   notation, thinking), plain Rust, tested natively
//! - the functions below: each answers with a short JSON text for the page
//! - `wasm`: the handful of functions the `.wasm` file exports, which only
//!   move text in and out of memory
//!
//! The JavaScript side is in the portfolio repo (`site/stargazer/`).

pub mod game;

use game::{Game, Thought};
use stargazer::board::Color;
use stargazer::search::mate_in;

/// The engine's name and version, as `uci` reports it.
pub fn version() -> String {
    format!("{} {}", stargazer::NAME, stargazer::VERSION)
}

/// Start a new game from a FEN (blank for the usual start). Answers with
/// the game's state (see `state`), or `{"error":"..."}`.
pub fn new_game(game: &mut Game, fen: &str) -> String {
    let fen = fen.trim();
    if fen.is_empty() {
        *game = Game::new();
    } else {
        match Game::from_fen(fen) {
            Ok(new) => *game = new,
            Err(problem) => return error(&problem),
        }
    }
    state(game)
}

/// Play a move written the UCI way (`e2e4`, `e7e8q`). Answers with the
/// move in chess notation plus the new state,
/// `{"san":"e4","fen":...}`, or `{"error":"..."}` if it isn't legal.
pub fn play(game: &mut Game, text: &str) -> String {
    match game.play(text.trim()) {
        Some(san) => format!("{{\"san\":{},{}", quote(&san), &state(game)[1..]),
        None => error(&format!("{} is not a legal move here", text.trim())),
    }
}

/// Think for about `millis` milliseconds and say which move to play,
/// without playing it: `{"move":"e2e4","depth":9,"score":"+0.31"}`. The
/// score is for the side to move: centipawns as pawns, or `#3` / `-#2`
/// for a forced mate. `{"error":"..."}` once the game is over.
pub fn think(game: &mut Game, millis: f64, now: &dyn Fn() -> f64) -> String {
    match game.think(millis, now) {
        Some(Thought { best, depth, score }) => format!(
            "{{\"move\":\"{best}\",\"depth\":{depth},\"score\":{}}}",
            quote(&score_text(score))
        ),
        None => error("the game is over"),
    }
}

/// Where the game stands:
/// `{"fen":"...","turn":"w","check":false,"status":"playing","legal":["e2e4",...]}`.
/// `status` is one of `game::State`'s names; `legal` is empty once the
/// game is over.
pub fn state(game: &Game) -> String {
    let position = game.position();
    let turn = match position.side_to_move {
        Color::White => "w",
        Color::Black => "b",
    };
    let legal: Vec<String> = game
        .legal_moves()
        .iter()
        .map(|mv| format!("\"{mv}\""))
        .collect();
    format!(
        "{{\"fen\":{},\"turn\":\"{turn}\",\"check\":{},\"status\":\"{}\",\"legal\":[{}]}}",
        quote(&position.to_fen()),
        position.in_check(position.side_to_move),
        game.state().name(),
        legal.join(",")
    )
}

/// `+0.31`, `-1.20`, `#3` (we mate in 3), `-#2` (we get mated in 2).
fn score_text(score: i32) -> String {
    match mate_in(score) {
        Some(moves) if moves > 0 => format!("#{moves}"),
        Some(moves) => format!("-#{}", -moves),
        None => format!("{:+.2}", score as f64 / 100.0),
    }
}

fn error(problem: &str) -> String {
    format!("{{\"error\":{}}}", quote(problem))
}

/// Text as a JSON string. Only FENs, move names and our own messages pass
/// through here, but quotes and backslashes are escaped anyway.
fn quote(text: &str) -> String {
    let mut quoted = String::from("\"");
    for letter in text.chars() {
        match letter {
            '"' => quoted.push_str("\\\""),
            '\\' => quoted.push_str("\\\\"),
            c if (c as u32) < 0x20 => quoted.push(' '),
            c => quoted.push(c),
        }
    }
    quoted.push('"');
    quoted
}

/// What the `.wasm` file exports. Text goes in through a buffer the page
/// asks for (`input`, then it writes the text there) and comes out through
/// another (each call returns the answer's length; `output` says where it
/// is). The page's clock comes in as the one imported function, `now`.
#[cfg(target_arch = "wasm32")]
mod wasm {
    use super::Game;
    use std::cell::RefCell;

    #[link(wasm_import_module = "env")]
    unsafe extern "C" {
        /// The page's clock, in milliseconds (`performance.now()`).
        fn now() -> f64;
    }

    thread_local! {
        static GAME: RefCell<Game> = RefCell::new(Game::new());
        static INPUT: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
        static OUTPUT: RefCell<String> = const { RefCell::new(String::new()) };
    }

    /// Make room for `length` bytes of text from the page; returns where
    /// to write them.
    #[unsafe(no_mangle)]
    pub extern "C" fn input(length: usize) -> *mut u8 {
        INPUT.with_borrow_mut(|input| {
            input.clear();
            input.resize(length, 0);
            input.as_mut_ptr()
        })
    }

    /// Where the last answer is.
    #[unsafe(no_mangle)]
    pub extern "C" fn output() -> *const u8 {
        OUTPUT.with_borrow(|output| output.as_ptr())
    }

    fn text_in() -> String {
        INPUT.with_borrow(|input| String::from_utf8_lossy(input).into_owned())
    }

    fn answer(text: String) -> usize {
        OUTPUT.with_borrow_mut(|output| {
            *output = text;
            output.len()
        })
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn version() -> usize {
        answer(super::version())
    }

    /// The input holds a FEN, or nothing for the usual start.
    #[unsafe(no_mangle)]
    pub extern "C" fn new_game() -> usize {
        let fen = text_in();
        answer(GAME.with_borrow_mut(|game| super::new_game(game, &fen)))
    }

    /// The input holds a move such as `e2e4`.
    #[unsafe(no_mangle)]
    pub extern "C" fn play() -> usize {
        let mv = text_in();
        answer(GAME.with_borrow_mut(|game| super::play(game, &mv)))
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn think(millis: f64) -> usize {
        // Safe: `now` is a plain function the page supplies; it takes and
        // returns a number.
        let clock = || unsafe { now() };
        answer(GAME.with_borrow_mut(|game| super::think(game, millis, &clock)))
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn state() -> usize {
        answer(GAME.with_borrow(super::state))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn the_state_of_a_new_game() {
        let mut game = Game::new();
        let text = new_game(&mut game, "");
        assert!(text.starts_with(
            "{\"fen\":\"rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1\",\"turn\":\"w\",\"check\":false,\"status\":\"playing\",\"legal\":[\""
        ));
        let legal = text.split("\"legal\":[").nth(1).unwrap();
        assert_eq!(legal.split(',').count(), 20, "{text}");
        assert!(text.contains("\"e2e4\"") && text.ends_with("\"]}"));
    }

    #[test]
    fn a_bad_fen_is_explained_and_the_game_kept() {
        let mut game = Game::new();
        play(&mut game, "e2e4");
        let text = new_game(&mut game, "not a position");
        assert!(text.starts_with("{\"error\":\""), "{text}");
        assert_eq!(game.position().to_fen().split(' ').nth(1), Some("b"));
    }

    #[test]
    fn playing_answers_with_the_notation_and_the_new_state() {
        let mut game = Game::new();
        let text = play(&mut game, "g1f3");
        assert!(text.starts_with("{\"san\":\"Nf3\",\"fen\":\""), "{text}");
        assert!(text.contains("\"turn\":\"b\""));
        assert_eq!(
            play(&mut game, "e2e4"),
            "{\"error\":\"e2e4 is not a legal move here\"}"
        );
    }

    #[test]
    fn checkmate_shows_in_the_state() {
        let mut game = Game::new();
        for mv in ["f2f3", "e7e5", "g2g4"] {
            play(&mut game, mv);
        }
        let text = play(&mut game, "d8h4");
        assert!(text.starts_with("{\"san\":\"Qh4#\""));
        assert!(text.ends_with("\"check\":true,\"status\":\"checkmate\",\"legal\":[]}"));
        let started = Instant::now();
        let now = move || started.elapsed().as_secs_f64() * 1000.0;
        assert_eq!(
            think(&mut game, 100.0, &now),
            "{\"error\":\"the game is over\"}"
        );
    }

    #[test]
    fn thinking_names_a_move_and_a_score() {
        let mut game = Game::new();
        new_game(&mut game, "6k1/5ppp/8/8/8/8/8/R5K1 w - - 0 1");
        let started = Instant::now();
        let now = move || started.elapsed().as_secs_f64() * 1000.0;
        let text = think(&mut game, 200.0, &now);
        assert!(
            text.starts_with("{\"move\":\"a1a8\",\"depth\":")
                && text.ends_with(",\"score\":\"#1\"}"),
            "{text}"
        );
    }

    #[test]
    fn scores_read_like_a_chess_site() {
        assert_eq!(score_text(31), "+0.31");
        assert_eq!(score_text(-120), "-1.20");
        assert_eq!(score_text(0), "+0.00");
        assert_eq!(score_text(stargazer::search::MATE - 5), "#3");
        assert_eq!(score_text(-(stargazer::search::MATE - 4)), "-#2");
    }

    #[test]
    fn quoting_escapes_what_json_needs() {
        assert_eq!(quote("a\"b\\c\n"), "\"a\\\"b\\\\c \"");
    }

    #[test]
    fn the_version_names_the_engine() {
        assert_eq!(version(), format!("Stargazer {}", stargazer::VERSION));
    }
}
