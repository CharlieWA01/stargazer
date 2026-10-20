//! UCI (Universal Chess Interface): the text protocol chess GUIs and
//! tournament tools use to talk to an engine over stdin/stdout.
//!
//! So far this handles the startup handshake, setting up a position
//! (`position startpos` or `position fen ...`, optionally followed by
//! `moves ...`), `go depth N` (search N half-moves deep and answer with
//! `bestmove`), `go perft N` (count move sequences), `d` (print the board)
//! and `eval` (print the evaluation). Clocks and `stop` come next
//! (ROADMAP.md, Milestone 2 step 3); until then, `go` without a depth
//! searches to `DEFAULT_DEPTH`.

use std::io::{BufRead, Write};
use std::time::Instant;

use crate::board::{Color, Position};
use crate::eval::evaluate;
use crate::movegen::legal_moves;
use crate::perft::divide;
use crate::search::{Report, mate_in, search};

/// How deep `go` searches when it isn't given a depth (until clocks are
/// handled). Takes well under a second in a release build.
const DEFAULT_DEPTH: u32 = 5;

/// Read UCI commands line by line and reply until `quit` or end of input.
pub fn run(input: impl BufRead, mut output: impl Write) {
    // Every position of the game so far, the current one last: the search
    // needs the earlier ones for the repetition rule.
    let mut game = vec![Position::startpos()];

    for line in input.lines() {
        let Ok(line) = line else { break };
        let command = line.split_whitespace().next().unwrap_or("");

        match command {
            "uci" => {
                writeln!(output, "id name {} {}", crate::NAME, crate::VERSION).ok();
                writeln!(output, "id author {}", crate::AUTHOR).ok();
                writeln!(output, "uciok").ok();
            }
            "isready" => {
                writeln!(output, "readyok").ok();
            }
            "ucinewgame" => game = vec![Position::startpos()],
            "position" => match parse_position(&line) {
                Ok(new_game) => game = new_game,
                // UCI has no error reply, so explain with an "info string"
                // and keep the old position.
                Err(reason) => {
                    writeln!(output, "info string invalid position: {reason}").ok();
                }
            },
            "go" => go(&line, &game, &mut output),
            // Not part of UCI, but most engines have it: print the board.
            "d" => {
                writeln!(output, "{}", current(&game)).ok();
            }
            // Also not UCI: the evaluation of the current position, without
            // looking ahead, shown from White's side like most tools do.
            "eval" => {
                let position = current(&game);
                let for_white = match position.side_to_move {
                    Color::White => evaluate(position),
                    Color::Black => -evaluate(position),
                };
                let pawns = f64::from(for_white) / 100.0;
                writeln!(output, "Evaluation: {pawns:+.2} (White's view)").ok();
            }
            "quit" => break,
            // The UCI spec says to ignore anything we don't understand.
            _ => {}
        }

        output.flush().ok();
    }
}

fn current(game: &[Position]) -> &Position {
    game.last().expect("the game always has a current position")
}

/// `go perft N` counts every move sequence N moves deep, listed by first
/// move (a debugging tool many engines share). Any other `go` searches,
/// printing an `info` line after each depth and then `bestmove`.
fn go(line: &str, game: &[Position], output: &mut impl Write) {
    let words: Vec<&str> = line.split_whitespace().skip(1).collect();
    let number_after = |word: &str| {
        let at = words.iter().position(|&w| w == word)?;
        words.get(at + 1)?.parse::<u32>().ok()
    };

    if words.first() == Some(&"perft") {
        match number_after("perft") {
            Some(depth) => {
                let split = divide(current(game), depth);
                for (mv, count) in &split {
                    writeln!(output, "{mv}: {count}").ok();
                }
                let total: u64 = split.iter().map(|(_, count)| count).sum();
                writeln!(output).ok();
                writeln!(output, "Nodes searched: {total}").ok();
            }
            None => {
                writeln!(output, "info string usage: go perft <depth>").ok();
            }
        }
        return;
    }

    let depth = number_after("depth").unwrap_or(DEFAULT_DEPTH);
    let started = Instant::now();
    let best = search(game, depth, &mut || false, &mut |report| {
        writeln!(output, "{}", info_line(report, started)).ok();
        output.flush().ok();
        true
    });
    match best {
        Some(mv) => writeln!(output, "bestmove {mv}").ok(),
        // No legal move (checkmate or stalemate): UCI's "null move".
        None => writeln!(output, "bestmove 0000").ok(),
    };
}

/// `info depth 5 score cp 30 nodes 77045 time 92 nps 837445 pv b1c3 ...`
fn info_line(report: &Report, started: Instant) -> String {
    let score = match mate_in(report.score) {
        Some(moves) => format!("mate {moves}"),
        None => format!("cp {}", report.score),
    };
    let millis = started.elapsed().as_millis() as u64;
    let nps = report.nodes * 1000 / millis.max(1);
    let pv: Vec<String> = report.pv.iter().map(|mv| mv.to_string()).collect();
    format!(
        "info depth {} score {score} nodes {} time {millis} nps {nps} pv {}",
        report.depth,
        report.nodes,
        pv.join(" ")
    )
}

/// Read a `position` command: `position startpos` or `position fen <FEN>`,
/// optionally followed by `moves e2e4 e7e5 ...`. Returns every position of
/// the game, the current one last.
fn parse_position(line: &str) -> Result<Vec<Position>, String> {
    let rest = line
        .trim_start()
        .strip_prefix("position")
        .unwrap_or("")
        .trim();
    let (setup, moves) = match rest.split_once("moves") {
        Some((setup, moves)) => (setup.trim(), moves.trim()),
        None => (rest, ""),
    };
    let mut position = if setup == "startpos" {
        Position::startpos()
    } else if let Some(fen) = setup.strip_prefix("fen") {
        Position::from_fen(fen.trim()).map_err(|e| e.to_string())?
    } else {
        return Err(format!("expected startpos or fen, got \"{setup}\""));
    };
    // Play each move in turn, accepting only legal ones.
    let mut game = vec![position];
    for text in moves.split_whitespace() {
        let mv = legal_moves(&position)
            .into_iter()
            .find(|mv| mv.to_string() == text)
            .ok_or_else(|| format!("{text} is not a legal move here"))?;
        position = position.make_move(mv);
        game.push(position);
    }
    Ok(game)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn respond(input: &str) -> String {
        let mut output = Vec::new();
        run(input.as_bytes(), &mut output);
        String::from_utf8(output).unwrap()
    }

    #[test]
    fn handshake() {
        let reply = respond("uci\n");
        assert!(reply.starts_with("id name Stargazer"));
        assert!(reply.ends_with("uciok\n"));
    }

    #[test]
    fn isready() {
        assert_eq!(respond("isready\n"), "readyok\n");
    }

    #[test]
    fn stops_at_quit() {
        assert_eq!(respond("quit\nisready\n"), "");
    }

    #[test]
    fn d_prints_the_start_position_by_default() {
        let reply = respond("d\n");
        assert!(reply.starts_with("8  r n b q k b n r\n"));
        assert!(reply.contains("FEN: rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"));
    }

    #[test]
    fn position_fen_sets_the_board() {
        let fen = "4k3/8/8/8/8/8/8/4K2R w K - 0 1";
        let reply = respond(&format!("position fen {fen}\nd\n"));
        assert!(reply.contains(&format!("FEN: {fen}")));
    }

    #[test]
    fn a_bad_position_is_explained_and_ignored() {
        let reply = respond("position fen 8/8/8 w - - 0 1\nd\n");
        assert!(reply.starts_with("info string invalid position: expected 8 ranks, found 3\n"));
        // The board is still the start position.
        assert!(reply.contains("FEN: rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"));
    }

    #[test]
    fn moves_after_a_position_are_played() {
        let reply = respond("position startpos moves e2e4 e7e5 g1f3\nd\n");
        assert!(
            reply.contains("FEN: rnbqkbnr/pppp1ppp/8/4p3/4P3/5N2/PPPP1PPP/RNBQKB1R b KQkq - 1 2")
        );
    }

    #[test]
    fn an_illegal_move_is_refused() {
        // After 1. e4, White can't move again.
        let reply = respond("position startpos moves e2e4 d2d4\n");
        assert_eq!(
            reply,
            "info string invalid position: d2d4 is not a legal move here\n"
        );
    }

    #[test]
    fn go_perft_lists_each_first_move_and_the_total() {
        let reply = respond("go perft 2\n");
        assert!(reply.contains("e2e4: 20\n"));
        assert!(reply.ends_with("Nodes searched: 400\n"));
    }

    #[test]
    fn ignores_unknown_commands() {
        assert_eq!(respond("xyzzy\nisready\n"), "readyok\n");
    }

    #[test]
    fn go_depth_searches_and_answers_with_bestmove() {
        // Back-rank mate in one.
        let reply = respond("position fen 6k1/5ppp/8/8/8/8/8/R5K1 w - - 0 1\ngo depth 2\n");
        assert!(reply.contains("info depth 1 score mate 1 "), "{reply}");
        assert!(reply.contains("info depth 2 "), "{reply}");
        assert!(reply.ends_with("bestmove a1a8\n"), "{reply}");
    }

    #[test]
    fn go_with_no_legal_moves_answers_the_null_move() {
        let reply = respond("position fen 7k/5Q2/6K1/8/8/8/8/8 b - - 0 1\ngo depth 3\n");
        assert_eq!(reply, "bestmove 0000\n");
    }

    #[test]
    fn eval_is_shown_from_whites_side() {
        assert_eq!(respond("eval\n"), "Evaluation: +0.00 (White's view)\n");
        // Black to move and a queen down: still positive, because it's
        // White who is better.
        let reply = respond("position fen 4k3/8/8/8/8/8/8/3QK3 b - - 0 1\neval\n");
        let pawns: f64 = reply
            .trim_start_matches("Evaluation: ")
            .split_whitespace()
            .next()
            .unwrap()
            .parse()
            .unwrap();
        assert!(pawns > 8.0, "{reply}");
    }
}
