//! UCI (Universal Chess Interface): the text protocol chess GUIs and
//! tournament tools use to talk to an engine over stdin/stdout.
//!
//! So far this handles the startup handshake, setting up a position
//! (`position startpos` or `position fen ...`), and `d`, which prints the
//! current board. It doesn't play moves yet.

use std::io::{BufRead, Write};

use crate::board::Position;

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
    let position = if setup == "startpos" {
        Position::startpos()
    } else if let Some(fen) = setup.strip_prefix("fen") {
        Position::from_fen(fen.trim()).map_err(|e| e.to_string())?
    } else {
        return Err(format!("expected startpos or fen, got \"{setup}\""));
    };
    if !moves.is_empty() {
        // Playing moves comes with move generation (Milestone 1, next steps).
        return Err("playing moves isn't supported yet".to_string());
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
    fn moves_are_not_supported_yet() {
        let reply = respond("position startpos moves e2e4\n");
        assert_eq!(
            reply,
            "info string invalid position: playing moves isn't supported yet\n"
        );
    }

    #[test]
    fn ignores_unknown_commands() {
        assert_eq!(respond("xyzzy\nisready\n"), "readyok\n");
    }
}
