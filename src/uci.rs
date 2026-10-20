//! UCI (Universal Chess Interface): the text protocol chess GUIs and
//! tournament tools use to talk to an engine over stdin/stdout.
//!
//! So far this handles the startup handshake, setting up a position
//! (`position startpos` or `position fen ...`, optionally followed by
//! `moves ...`), `d`, which prints the current board, and `go perft N`,
//! which counts move sequences. It doesn't search for a best move yet.

use std::io::{BufRead, Write};

use crate::board::Position;
use crate::movegen::legal_moves;
use crate::perft::divide;

/// Read UCI commands line by line and reply until `quit` or end of input.
pub fn run(input: impl BufRead, mut output: impl Write) {
    let mut position = Position::startpos();

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
            "ucinewgame" => position = Position::startpos(),
            "position" => match parse_position(&line) {
                Ok(new_position) => position = new_position,
                // UCI has no error reply, so explain with an "info string"
                // and keep the old position.
                Err(reason) => {
                    writeln!(output, "info string invalid position: {reason}").ok();
                }
            },
            // `go perft N`: count every move sequence N moves deep, listed by
            // first move (a debugging tool many engines share).
            "go" => {
                let mut words = line.split_whitespace().skip(1);
                if words.next() == Some("perft") {
                    match words.next().and_then(|n| n.parse().ok()) {
                        Some(depth) => {
                            let split = divide(&position, depth);
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
                }
            }
            // Not part of UCI, but most engines have it: print the board.
            "d" => {
                writeln!(output, "{position}").ok();
            }
            "quit" => break,
            // The UCI spec says to ignore anything we don't understand.
            _ => {}
        }

        output.flush().ok();
    }
}

/// Read a `position` command: `position startpos` or `position fen <FEN>`,
/// optionally followed by `moves e2e4 e7e5 ...`.
fn parse_position(line: &str) -> Result<Position, String> {
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
    for text in moves.split_whitespace() {
        let mv = legal_moves(&position)
            .into_iter()
            .find(|mv| mv.to_string() == text)
            .ok_or_else(|| format!("{text} is not a legal move here"))?;
        position = position.make_move(mv);
    }
    Ok(position)
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
}
