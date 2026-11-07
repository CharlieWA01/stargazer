//! A game in progress, as a web page sees it: the moves so far, whose turn
//! it is, which moves are legal, whether the game is over, and the
//! engine's choice of move. The chess rules all come from the engine; the
//! page never needs its own.

use stargazer::board::{Color, Piece, PieceKind, Position};
use stargazer::movegen::{Move, legal_moves};
use stargazer::search::{History, MAX_PLY, Report, search};
use stargazer::tt::{DEFAULT_MB, Table};

/// How a game stands.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum State {
    Playing,
    Checkmate,
    Stalemate,
    /// The same position for the third time (drawn at once, as Lichess does).
    Repetition,
    /// 50 moves each without a capture or a pawn move.
    FiftyMoves,
    /// Neither side has enough pieces left to checkmate.
    InsufficientMaterial,
}

impl State {
    /// The name the page uses.
    pub fn name(self) -> &'static str {
        match self {
            State::Playing => "playing",
            State::Checkmate => "checkmate",
            State::Stalemate => "stalemate",
            State::Repetition => "repetition",
            State::FiftyMoves => "fifty-moves",
            State::InsufficientMaterial => "insufficient-material",
        }
    }
}

/// What the engine decided after thinking.
#[derive(Clone, Debug, PartialEq)]
pub struct Thought {
    pub best: Move,
    /// The last depth it finished (0 when there was only one legal move).
    pub depth: u32,
    /// Centipawns for the side that moves, or near `MATE` for a forced mate.
    pub score: i32,
}

pub struct Game {
    /// Every position so far, the current one last (the search needs them
    /// to spot repeats).
    positions: Vec<Position>,
    /// The search's memory, kept between moves as in the native engine.
    table: Table,
    history: History,
}

impl Game {
    /// A new game from the usual starting position.
    pub fn new() -> Game {
        Game::starting_from(Position::startpos())
    }

    /// A new game from a position written as FEN.
    pub fn from_fen(fen: &str) -> Result<Game, String> {
        let position = Position::from_fen(fen).map_err(|e| e.to_string())?;
        Ok(Game::starting_from(position))
    }

    fn starting_from(position: Position) -> Game {
        Game {
            positions: vec![position],
            // The same table size as the native engine's default (16 MB).
            table: Table::new(DEFAULT_MB),
            history: History::new(),
        }
    }

    pub fn position(&self) -> &Position {
        self.positions.last().expect("a game always has a position")
    }

    /// The legal moves, none once the game is over (a draw ends the game
    /// even when moves are left).
    pub fn legal_moves(&self) -> Vec<Move> {
        if self.state() == State::Playing {
            legal_moves(self.position())
        } else {
            Vec::new()
        }
    }

    pub fn state(&self) -> State {
        let position = self.position();
        // Checkmate comes first: a mate on the 50th move still counts.
        if legal_moves(position).is_empty() {
            return if position.in_check(position.side_to_move) {
                State::Checkmate
            } else {
                State::Stalemate
            };
        }
        if self.times_seen(position) >= 3 {
            State::Repetition
        } else if position.halfmove_clock >= 100 {
            State::FiftyMoves
        } else if position.insufficient_material() {
            State::InsufficientMaterial
        } else {
            State::Playing
        }
    }

    /// How many times this position has stood on the board, counting now.
    fn times_seen(&self, position: &Position) -> usize {
        self.positions
            .iter()
            .filter(|earlier| earlier.repeats(position))
            .count()
    }

    /// Play a move written the UCI way (`e2e4`, `e7e8q`). Returns it in
    /// the usual chess notation (`e4`, `e8=Q+`), or `None` if it isn't
    /// legal here.
    pub fn play(&mut self, text: &str) -> Option<String> {
        let mv = self
            .legal_moves()
            .into_iter()
            .find(|mv| mv.to_string() == text)?;
        let notation = san(self.position(), mv);
        let next = self.position().make_move(mv);
        self.positions.push(next);
        Some(notation)
    }

    /// Think for about `millis` milliseconds and say which move to play
    /// (without playing it). `now` is the page's clock in milliseconds: the
    /// engine core never reads a clock itself. `None` once the game is over.
    pub fn think(&mut self, millis: f64, now: &dyn Fn() -> f64) -> Option<Thought> {
        let moves = self.legal_moves();
        match moves.as_slice() {
            [] => return None,
            // Only one legal move: nothing to think about.
            [only] => {
                return Some(Thought {
                    best: *only,
                    depth: 0,
                    score: 0,
                });
            }
            _ => {}
        }
        let started = now();
        let mut last: Option<Report> = None;
        let best = search(
            &self.positions,
            MAX_PLY as u32,
            &mut self.table,
            &mut self.history,
            // Inside a depth: stop at the time limit.
            &mut || now() - started >= millis,
            // Between depths: each takes several times longer than the
            // last, so past half the time another one won't finish (the
            // same rule as the native engine's `go movetime`).
            &mut |report| {
                last = Some(report.clone());
                now() - started < millis / 2.0
            },
        )?;
        let (depth, score) = last.map_or((0, 0), |r| (r.depth, r.score));
        Some(Thought { best, depth, score })
    }
}

impl Default for Game {
    fn default() -> Game {
        Game::new()
    }
}

/// A move in standard algebraic notation, as chess players write it:
/// `Nf3`, `exd5`, `O-O`, `Rad1`, `e8=Q#`. `mv` must be legal in `position`.
pub fn san(position: &Position, mv: Move) -> String {
    let piece = position
        .piece_on(mv.from)
        .expect("a legal move starts on a piece");
    let capture = position.piece_on(mv.to).is_some()
        // En passant: a pawn moving diagonally onto an empty square.
        || (piece.kind == PieceKind::Pawn && mv.from.file() != mv.to.file());

    let mut text = String::new();
    if piece.kind == PieceKind::King && mv.from.file().abs_diff(mv.to.file()) == 2 {
        // Castling: the king moves two squares, towards the h-file (short)
        // or the a-file (long).
        text.push_str(if mv.to.file() > mv.from.file() {
            "O-O"
        } else {
            "O-O-O"
        });
    } else if piece.kind == PieceKind::Pawn {
        // Pawns are named only by their file, and only when capturing.
        if capture {
            text.push((b'a' + mv.from.file()) as char);
            text.push('x');
        }
        text.push_str(&mv.to.to_string());
        if let Some(kind) = mv.promotion {
            text.push('=');
            text.push(Piece::new(Color::White, kind).to_char());
        }
    } else {
        text.push(Piece::new(Color::White, piece.kind).to_char());
        text.push_str(&disambiguation(position, mv, piece));
        if capture {
            text.push('x');
        }
        text.push_str(&mv.to.to_string());
    }

    // Check (+) or checkmate (#).
    let next = position.make_move(mv);
    if next.in_check(next.side_to_move) {
        text.push(if legal_moves(&next).is_empty() {
            '#'
        } else {
            '+'
        });
    }
    text
}

/// When two pieces of the same kind could go to the same square, say which
/// one moved: by its file if that's enough (`Nbd2`), else by its rank
/// (`R1e2`), else both (`Qh4e1`).
fn disambiguation(position: &Position, mv: Move, piece: Piece) -> String {
    let rivals: Vec<Move> = legal_moves(position)
        .into_iter()
        .filter(|other| {
            other.to == mv.to
                && other.from != mv.from
                && position.piece_on(other.from) == Some(piece)
        })
        .collect();
    if rivals.is_empty() {
        return String::new();
    }
    let square = mv.from.to_string();
    let (file, rank) = square.split_at(1);
    if rivals.iter().all(|r| r.from.file() != mv.from.file()) {
        file.to_string()
    } else if rivals.iter().all(|r| r.from.rank() != mv.from.rank()) {
        rank.to_string()
    } else {
        square
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    fn game(fen: &str) -> Game {
        Game::from_fen(fen).unwrap()
    }

    fn clock() -> impl Fn() -> f64 {
        let started = Instant::now();
        move || started.elapsed().as_secs_f64() * 1000.0
    }

    /// Play UCI moves in turn, returning their notation.
    fn play_all(game: &mut Game, moves: &str) -> Vec<String> {
        moves
            .split_whitespace()
            .map(|mv| game.play(mv).unwrap_or_else(|| panic!("{mv} refused")))
            .collect()
    }

    #[test]
    fn the_start_position_has_twenty_moves_and_play_goes_on() {
        let game = Game::new();
        assert_eq!(game.legal_moves().len(), 20);
        assert_eq!(game.state(), State::Playing);
    }

    #[test]
    fn an_illegal_move_is_refused_and_changes_nothing() {
        let mut game = Game::new();
        assert_eq!(game.play("e2e5"), None);
        assert_eq!(game.play("nonsense"), None);
        assert_eq!(game.position(), &Position::startpos());
    }

    #[test]
    fn fools_mate_is_checkmate() {
        let mut game = Game::new();
        let notation = play_all(&mut game, "f2f3 e7e5 g2g4 d8h4");
        assert_eq!(notation, ["f3", "e5", "g4", "Qh4#"]);
        assert_eq!(game.state(), State::Checkmate);
        assert!(game.legal_moves().is_empty());
        assert_eq!(game.play("a2a3"), None, "no moves after the game ends");
    }

    #[test]
    fn stalemate() {
        // Black to move, not in check, and nowhere to go.
        let game = game("7k/5Q2/6K1/8/8/8/8/8 b - - 0 1");
        assert_eq!(game.state(), State::Stalemate);
    }

    #[test]
    fn the_third_repetition_is_a_draw() {
        let mut game = Game::new();
        // The knights go out and back twice: the start position appears a
        // third time.
        play_all(&mut game, "g1f3 g8f6 f3g1 f6g8 g1f3 g8f6 f3g1");
        assert_eq!(game.state(), State::Playing);
        game.play("f6g8").unwrap();
        assert_eq!(game.state(), State::Repetition);
        assert!(game.legal_moves().is_empty());
    }

    #[test]
    fn fifty_moves_without_a_capture_or_pawn_move_is_a_draw() {
        let game = game("4k3/8/8/8/8/8/R7/4K3 w - - 100 80");
        assert_eq!(game.state(), State::FiftyMoves);
    }

    #[test]
    fn checkmate_on_the_fiftieth_move_still_counts() {
        // White's rook mates with the clock at 99: the mate stands.
        let mut game = game("k7/8/1K6/8/8/8/8/7R w - - 99 80");
        assert_eq!(game.play("h1h8").as_deref(), Some("Rh8#"));
        assert_eq!(game.state(), State::Checkmate);
    }

    #[test]
    fn king_against_king_is_a_draw() {
        let game = game("4k3/8/8/8/8/8/8/4K3 w - - 0 1");
        assert_eq!(game.state(), State::InsufficientMaterial);
    }

    #[test]
    fn notation_for_captures_castling_promotion_and_en_passant() {
        let mut short = Game::new();
        let notation = play_all(
            &mut short,
            "e2e4 d7d5 e4d5 g8f6 g1f3 c7c5 d5c6 f6e4 f1b5 c8d7 e1g1 b8a6 c6b7 d8c7 b7a8q",
        );
        assert_eq!(
            notation,
            [
                "e4", "d5", "exd5", "Nf6", "Nf3", "c5", "dxc6", "Ne4", "Bb5", "Bd7", "O-O", "Na6",
                "cxb7", "Qc7", "bxa8=Q+",
            ]
        );
        // Long castling.
        let mut long = game("r3k3/8/8/8/8/8/8/4K3 b q - 0 1");
        assert_eq!(long.play("e8c8").as_deref(), Some("O-O-O"));
    }

    #[test]
    fn notation_says_which_piece_moved_when_two_could() {
        // Knights on b1 and f3 can both reach d2: named by file.
        let position = Position::from_fen("4k3/8/8/8/8/5N2/8/1N2K3 w - - 0 1").unwrap();
        let mv = Move::new(
            stargazer::board::Square::parse("b1").unwrap(),
            stargazer::board::Square::parse("d2").unwrap(),
        );
        assert_eq!(san(&position, mv), "Nbd2");
        // Rooks on a1 and a5 can both reach a3: same file, so by rank.
        let position = Position::from_fen("4k3/8/8/R7/8/8/8/R3K3 w - - 0 1").unwrap();
        let mv = Move::new(
            stargazer::board::Square::parse("a1").unwrap(),
            stargazer::board::Square::parse("a3").unwrap(),
        );
        assert_eq!(san(&position, mv), "R1a3");
        // Queens on a1, a5 and e1 can all reach e5. The one on a1 shares
        // its file with a5 and its rank with e1, so the whole square.
        let position = Position::from_fen("8/7k/8/Q7/8/8/8/Q3Q1K1 w - - 0 1").unwrap();
        let mv = Move::new(
            stargazer::board::Square::parse("a1").unwrap(),
            stargazer::board::Square::parse("e5").unwrap(),
        );
        assert_eq!(san(&position, mv), "Qa1e5");
    }

    #[test]
    fn it_finds_a_mate_in_one_and_keeps_to_its_time() {
        // White mates with Ra8.
        let mut game = game("6k1/5ppp/8/8/8/8/8/R5K1 w - - 0 1");
        let now = clock();
        let started = Instant::now();
        let thought = game.think(300.0, &now).unwrap();
        assert_eq!(thought.best.to_string(), "a1a8");
        assert!(thought.depth >= 1);
        // Well inside a generous margin (the search checks the clock every
        // few thousand positions).
        assert!(started.elapsed().as_millis() < 1000);
    }

    #[test]
    fn a_lone_legal_move_is_chosen_without_thinking() {
        // Black's king on a8: White's king guards a7 and b7, so only b8.
        let mut game = game("k7/8/1K6/8/8/8/8/7R b - - 0 1");
        assert_eq!(game.legal_moves().len(), 1);
        let thought = game.think(10_000.0, &clock()).unwrap();
        assert_eq!(thought.depth, 0);
        assert_eq!(thought.best.to_string(), "a8b8");
    }

    #[test]
    fn nothing_to_think_about_once_the_game_is_over() {
        let mut game = game("7k/5Q2/6K1/8/8/8/8/8 b - - 0 1");
        assert_eq!(game.think(100.0, &clock()), None);
    }

    #[test]
    fn a_whole_game_against_itself_ends_properly() {
        // Quick moves from the start until the game ends or 300 half-moves
        // pass; every move it picks must be legal.
        let mut game = Game::new();
        let now = clock();
        for _ in 0..300 {
            let Some(thought) = game.think(5.0, &now) else {
                break;
            };
            assert!(game.play(&thought.best.to_string()).is_some());
        }
        assert!(game.positions.len() > 10);
    }
}
