//! UCI (Universal Chess Interface): the text protocol chess GUIs and
//! tournament tools use to talk to an engine over stdin/stdout.
//!
//! For now this only handles the startup handshake, so GUIs can recognize
//! the engine. It doesn't play moves yet.

use std::io::{BufRead, Write};

/// Read UCI commands line by line and reply until `quit` or end of input.
pub fn run(input: impl BufRead, mut output: impl Write) {
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
            "quit" => break,
            // The UCI spec says to ignore anything we don't understand.
            _ => {}
        }

        output.flush().ok();
    }
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
    fn ignores_unknown_commands() {
        assert_eq!(respond("xyzzy\nisready\n"), "readyok\n");
    }
}
