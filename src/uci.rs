//! UCI (Universal Chess Interface): the text protocol chess GUIs and
//! tournament tools use to talk to an engine over stdin/stdout.
//!
//! Handles the startup handshake, `position startpos` or `position fen ...`
//! (optionally followed by `moves ...`), `go` with a clock (`wtime btime
//! winc binc movestogo`), `go movetime N`, `go depth N` and `go infinite`,
//! `stop`, `quit`, and two options: `setoption name Hash value <MB>` (the
//! transposition table's size, 1 to 1024 MB) and `setoption name Move
//! Overhead value <ms>` (time kept back on every move for the GUI or
//! server to pass it on). With only one legal move and a clock running,
//! the move is played at once. Extras most engines share: `go perft N` (count move
//! sequences), `d` (print the board), `eval` (print the evaluation) and
//! `bench` (a fixed search; its node count fingerprints the build).
//!
//! The search runs on its own thread, so this loop keeps reading commands
//! while it thinks: `stop` and `isready` are answered straight away.

use std::fmt::Display;
use std::io::{BufRead, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use crate::board::{Color, Position};
use crate::eval::{evaluate, game_phase};
use crate::movegen::legal_moves;
use crate::perft::divide;
use crate::search::{History, MAX_PLY, Report, mate_in, search};
use crate::tt::{self, Table};

/// How deep a bare `go` (no clock, depth or time) searches. Handy when
/// typing commands by hand; well under a second in a release build.
const DEFAULT_DEPTH: u32 = 5;

/// Time kept back on every move for the GUI or Lichess to pass the move on,
/// so the engine never loses on time by a whisker: the `Move Overhead`
/// option's default, and its limit.
const DEFAULT_MOVE_OVERHEAD_MS: u64 = 50;
const MAX_MOVE_OVERHEAD_MS: u64 = 5_000;

/// Output shared between this loop and the search thread.
type Output<W> = Arc<Mutex<W>>;

/// What the search remembers from move to move (the transposition table
/// and the history scores), lent to each search.
struct Memory {
    table: Table,
    history: History,
}

type SharedMemory = Arc<Mutex<Memory>>;

/// The `Hash` option's limits, in megabytes.
const MAX_HASH_MB: usize = 1024;

/// Print one line and send it at once (GUIs wait for whole lines).
fn say<W: Write>(output: &Output<W>, text: impl Display) {
    let mut output = output.lock().unwrap();
    writeln!(output, "{text}").ok();
    output.flush().ok();
}

/// A search running on its own thread.
struct Thinking {
    stop: Arc<AtomicBool>,
    thread: JoinHandle<()>,
    infinite: bool,
}

impl Thinking {
    /// Tell the search to stop, and wait until it has said `bestmove`.
    fn stop(self) {
        self.stop.store(true, Ordering::Relaxed);
        self.thread.join().ok();
    }
}

/// Read UCI commands line by line and reply until `quit` or end of input.
pub fn run<W: Write + Send + 'static>(input: impl BufRead, output: W) {
    let output = Arc::new(Mutex::new(output));
    // Every position of the game so far, the current one last: the search
    // needs the earlier ones for the repetition rule.
    let mut game = vec![Position::startpos()];
    let mut thinking: Option<Thinking> = None;
    let mut move_overhead = DEFAULT_MOVE_OVERHEAD_MS;
    let memory: SharedMemory = Arc::new(Mutex::new(Memory {
        table: Table::new(tt::DEFAULT_MB),
        history: History::new(),
    }));

    for line in input.lines() {
        let Ok(line) = line else { break };
        let command = line.split_whitespace().next().unwrap_or("");

        // A GUI shouldn't change the position or start a new search while
        // we think, but if one does, finish the current search first.
        if matches!(
            command,
            "ucinewgame" | "position" | "go" | "stop" | "quit" | "setoption"
        ) && let Some(search) = thinking.take()
        {
            search.stop();
        }

        match command {
            "uci" => {
                say(
                    &output,
                    format!("id name {} {}", crate::NAME, crate::VERSION),
                );
                say(&output, format!("id author {}", crate::AUTHOR));
                say(
                    &output,
                    format!(
                        "option name Hash type spin default {} min 1 max {MAX_HASH_MB}",
                        tt::DEFAULT_MB
                    ),
                );
                say(
                    &output,
                    format!(
                        "option name Move Overhead type spin default {DEFAULT_MOVE_OVERHEAD_MS} min 0 max {MAX_MOVE_OVERHEAD_MS}"
                    ),
                );
                say(&output, "uciok");
            }
            "isready" => say(&output, "readyok"),
            "ucinewgame" => {
                game = vec![Position::startpos()];
                // A new game: last game's positions and moves are no use.
                let mut memory = memory.lock().unwrap();
                memory.table.clear();
                memory.history.clear();
            }
            "setoption" => match parse_option(&line) {
                Some(EngineOption::Hash(megabytes)) => {
                    memory.lock().unwrap().table = Table::new(megabytes)
                }
                Some(EngineOption::MoveOverhead(ms)) => move_overhead = ms,
                None => say(
                    &output,
                    "info string unknown option (Hash and Move Overhead are supported)",
                ),
            },
            "position" => match parse_position(&line) {
                Ok(new_game) => game = new_game,
                // UCI has no error reply, so explain with an "info string"
                // and keep the old position.
                Err(reason) => say(&output, format!("info string invalid position: {reason}")),
            },
            "go" => thinking = go(&line, &game, &memory, &output, move_overhead),
            // Not part of UCI, but most engines have it: print the board.
            "d" => say(&output, current(&game)),
            // Also not UCI: the evaluation of the current position, without
            // looking ahead, shown from White's side like most tools do.
            "eval" => {
                let position = current(&game);
                let for_white = match position.side_to_move {
                    Color::White => evaluate(position),
                    Color::Black => -evaluate(position),
                };
                let pawns = f64::from(for_white) / 100.0;
                say(&output, format!("Evaluation: {pawns:+.2} (White's view)"));
            }
            // `bench` or `bench <depth>`: see `bench.rs`.
            "bench" => {
                let depth = line
                    .split_whitespace()
                    .nth(1)
                    .and_then(|d| d.parse().ok())
                    .unwrap_or(crate::bench::BENCH_DEPTH);
                say(&output, bench_report(depth));
            }
            "quit" => return,
            // `stop` was handled above. The UCI spec says to ignore anything
            // we don't understand.
            _ => {}
        }
    }

    // End of input (commands piped in from a file, say): let a search with
    // a limit finish; stop one that would run forever.
    if let Some(search) = thinking {
        if search.infinite {
            search.stop();
        } else {
            search.thread.join().ok();
        }
    }
}

/// `bench` as one line, in the form testing tools read:
/// `Bench: 2457379 nodes 569000 nps`.
pub fn bench_report(depth: u32) -> String {
    let result = crate::bench::run(depth);
    format!(
        "Bench: {} nodes {} nps",
        result.nodes,
        result.nodes_per_second()
    )
}

fn current(game: &[Position]) -> &Position {
    game.last().expect("the game always has a current position")
}

/// What a `go` command asked for.
#[derive(Default, Debug, PartialEq)]
struct GoLimits {
    depth: Option<u32>,
    movetime: Option<u64>,
    infinite: bool,
    /// Milliseconds left on each clock, and added per move: [White, Black].
    time: [Option<u64>; 2],
    increment: [u64; 2],
    moves_to_go: Option<u64>,
}

fn parse_go(line: &str) -> GoLimits {
    let words: Vec<&str> = line.split_whitespace().skip(1).collect();
    let number_after = |word: &str| -> Option<u64> {
        let at = words.iter().position(|&w| w == word)?;
        // Some GUIs send a negative time when the clock has run out.
        let value: i64 = words.get(at + 1)?.parse().ok()?;
        Some(value.max(0) as u64)
    };
    GoLimits {
        depth: number_after("depth").map(|d| d as u32),
        movetime: number_after("movetime"),
        infinite: words.contains(&"infinite"),
        time: [number_after("wtime"), number_after("btime")],
        increment: [
            number_after("winc").unwrap_or(0),
            number_after("binc").unwrap_or(0),
        ],
        moves_to_go: number_after("movestogo"),
    }
}

/// Start another depth only while under this share of the target, in
/// percent.
const NEW_DEPTH_PERCENT: u64 = 60;

/// How long to think, in milliseconds: a target (`soft`) and a ceiling
/// never crossed (`hard`).
#[derive(Debug, PartialEq)]
struct TimeBudget {
    soft: u64,
    hard: u64,
}

impl TimeBudget {
    /// Inside a depth: past the ceiling, stop now.
    fn out_of_time(&self, elapsed_ms: u64) -> bool {
        elapsed_ms >= self.hard
    }

    /// Between depths: each depth takes about twice as long as the one
    /// before (late move reductions keep it that low), so past 60% of the
    /// target there's little point starting another. `percent` stretches
    /// the target for this position (100: as planned), never past the
    /// ceiling.
    fn worth_another_depth(&self, elapsed_ms: u64, percent: u64) -> bool {
        let target = (self.soft * percent / 100).min(self.hard);
        elapsed_ms < target * NEW_DEPTH_PERCENT / 100
    }
}

/// The score each depth gave, to notice when the search has just found a
/// problem.
#[derive(Default)]
struct ScoreWatch {
    last: Option<i32>,
}

/// A fall in score, from one depth to the next, big enough to mean the
/// search has just found a problem: about a third of a pawn.
const SCORE_DROP_CP: i32 = 30;

impl ScoreWatch {
    /// After a depth, with that depth's score: how much of the target to
    /// allow, in percent. If the score has fallen by more than a third of
    /// a pawn since the last depth, the search has just seen trouble
    /// coming: twice the target, to look for a way out. Otherwise as
    /// planned; a rising score changes nothing.
    fn after_depth(&mut self, score: i32) -> u64 {
        let dropped = self
            .last
            .is_some_and(|last| score < last.saturating_sub(SCORE_DROP_CP));
        self.last = Some(score);
        if dropped { 200 } else { 100 }
    }
}

/// Plan this move's time (DESIGN.md section 3): about a twentieth of the
/// time left (or a share of it until the next time control) plus most of
/// the increment, and never more than three times that. With no increment
/// and no time control coming (sudden death), the clock never refills:
/// share it over the moves the game probably has left instead, judged
/// from the pieces still on the board (`sudden_death_moves_left`), and
/// never plan more than a tenth of it.
/// Returns `None` when there's no clock or move time, so the search has
/// no time limit.
fn time_budget(
    limits: &GoLimits,
    side: Color,
    phase: i32,
    move_overhead: u64,
) -> Option<TimeBudget> {
    if let Some(movetime) = limits.movetime {
        let hard = movetime.saturating_sub(move_overhead).max(1);
        return Some(TimeBudget { soft: hard, hard });
    }
    let index = match side {
        Color::White => 0,
        Color::Black => 1,
    };
    let remaining = limits.time[index]?;
    let increment = limits.increment[index];
    let available = remaining.saturating_sub(move_overhead).max(1);
    let sudden_death = limits.moves_to_go.is_none() && increment == 0;
    let share = match limits.moves_to_go {
        Some(moves) => (moves + 1).min(20),
        None if sudden_death => sudden_death_moves_left(phase),
        None => 20,
    };
    let soft = (remaining / share + increment * 3 / 4).min(available);
    let mut hard = (soft * 3).min(available);
    if sudden_death {
        hard = hard.min(remaining / 10);
    }
    Some(TimeBudget { soft, hard })
}

/// In sudden death, how many more moves the game probably lasts, judged
/// from the pieces left (the evaluation's game phase, 24 with every piece
/// on and 0 with only kings and pawns): 25 moves plus the phase, at most
/// 45. So the engine thinks hardest when the pieces come off and the game
/// turns into an endgame, and keeps a reserve while the board stays full
/// (Charlie's choice, 2026-10-25).
fn sudden_death_moves_left(phase: i32) -> u64 {
    (25 + phase.clamp(0, 20)) as u64
}

/// An option a GUI can set.
#[derive(Debug, PartialEq)]
enum EngineOption {
    /// The table size in megabytes, kept between 1 and 1024.
    Hash(usize),
    /// Milliseconds kept back on every move, kept between 0 and 5000.
    MoveOverhead(u64),
}

/// `setoption name Hash value 64` or `setoption name Move Overhead value
/// 100` (option names ignore case and may have spaces). `None` for any
/// other option or a bad value.
fn parse_option(line: &str) -> Option<EngineOption> {
    let words: Vec<&str> = line.split_whitespace().collect();
    let value_at = words.iter().position(|&w| w == "value")?;
    if words.get(1) != Some(&"name") || value_at != words.len() - 2 {
        return None;
    }
    let name = words[2..value_at].join(" ").to_lowercase();
    let value = words[value_at + 1];
    match name.as_str() {
        "hash" => value
            .parse()
            .ok()
            .map(|mb: usize| EngineOption::Hash(mb.clamp(1, MAX_HASH_MB))),
        "move overhead" => value
            .parse()
            .ok()
            .map(|ms: u64| EngineOption::MoveOverhead(ms.min(MAX_MOVE_OVERHEAD_MS))),
        _ => None,
    }
}

/// `go perft N` counts every move sequence N moves deep, listed by first
/// move (a debugging tool many engines share). Any other `go` starts a
/// search on its own thread, which prints an `info` line after each depth
/// and then `bestmove`.
fn go<W: Write + Send + 'static>(
    line: &str,
    game: &[Position],
    memory: &SharedMemory,
    output: &Output<W>,
    move_overhead: u64,
) -> Option<Thinking> {
    let words: Vec<&str> = line.split_whitespace().skip(1).collect();
    if words.first() == Some(&"perft") {
        match words.get(1).and_then(|n| n.parse().ok()) {
            Some(depth) => {
                let split = divide(current(game), depth);
                for (mv, count) in &split {
                    say(output, format!("{mv}: {count}"));
                }
                let total: u64 = split.iter().map(|(_, count)| count).sum();
                say(output, "");
                say(output, format!("Nodes searched: {total}"));
            }
            None => say(output, "info string usage: go perft <depth>"),
        }
        return None;
    }

    let limits = parse_go(line);
    let position = current(game);
    let budget = time_budget(
        &limits,
        position.side_to_move,
        game_phase(position),
        move_overhead,
    );
    // Only one legal move and a clock running: nothing to think about, so
    // save the time. (`go depth` and `go infinite` still search, for
    // anyone who wants to see the analysis.)
    let moves = legal_moves(current(game));
    if let [only] = moves.as_slice()
        && budget.is_some()
        && !limits.infinite
        && limits.depth.is_none()
    {
        say(output, format!("bestmove {only}"));
        return None;
    }
    let max_depth = match (limits.depth, limits.infinite || budget.is_some()) {
        (Some(depth), _) => depth,
        (None, true) => MAX_PLY as u32,
        (None, false) => DEFAULT_DEPTH,
    };
    let infinite = limits.infinite;

    let stop = Arc::new(AtomicBool::new(false));
    let game = game.to_vec();
    let memory = Arc::clone(memory);
    let output = Arc::clone(output);
    let thread_stop = Arc::clone(&stop);
    let thread = thread::spawn(move || {
        let started = Instant::now();
        let stop = thread_stop;
        let elapsed = || started.elapsed().as_millis() as u64;
        let mut memory = memory.lock().unwrap();
        let Memory { table, history } = &mut *memory;
        let mut scores = ScoreWatch::default();
        let best = search(
            &game,
            max_depth,
            table,
            history,
            // Inside a depth: stop when told to, or at the hard limit.
            &mut || {
                stop.load(Ordering::Relaxed)
                    || budget.as_ref().is_some_and(|b| b.out_of_time(elapsed()))
            },
            // Between depths: report, then decide whether to go deeper,
            // allowing more time when the score has just fallen.
            &mut |report| {
                say(&output, info_line(report, started));
                let percent = scores.after_depth(report.score);
                !stop.load(Ordering::Relaxed)
                    && budget
                        .as_ref()
                        .is_none_or(|b| b.worth_another_depth(elapsed(), percent))
            },
        );
        // `go infinite` must not answer until told to stop, even if the
        // search has run out of depths.
        while infinite && !stop.load(Ordering::Relaxed) {
            thread::sleep(Duration::from_millis(5));
        }
        match best {
            Some(mv) => say(&output, format!("bestmove {mv}")),
            // No legal move (checkmate or stalemate): UCI's "null move".
            None => say(&output, "bestmove 0000"),
        }
    });
    Some(Thinking {
        stop,
        thread,
        infinite,
    })
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

    /// Collects what the engine prints, from either thread.
    #[derive(Clone, Default)]
    struct Captured(Arc<Mutex<Vec<u8>>>);

    impl Write for Captured {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.0.lock().unwrap().write(bytes)
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    fn respond(input: &str) -> String {
        let captured = Captured::default();
        run(input.as_bytes(), captured.clone());
        String::from_utf8(captured.0.lock().unwrap().clone()).unwrap()
    }

    /// Input that arrives a line at a time, each after a pause, like a GUI
    /// typing commands while the engine thinks.
    struct Paced {
        lines: std::collections::VecDeque<(u64, String)>,
        pending: Vec<u8>,
    }

    impl std::io::Read for Paced {
        fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
            if self.pending.is_empty() {
                let Some((pause_ms, line)) = self.lines.pop_front() else {
                    return Ok(0);
                };
                thread::sleep(Duration::from_millis(pause_ms));
                self.pending = format!("{line}\n").into_bytes();
            }
            let n = buffer.len().min(self.pending.len());
            buffer[..n].copy_from_slice(&self.pending[..n]);
            self.pending.drain(..n);
            Ok(n)
        }
    }

    fn respond_paced(lines: &[(u64, &str)]) -> String {
        let input = Paced {
            lines: lines.iter().map(|&(ms, l)| (ms, l.to_string())).collect(),
            pending: Vec::new(),
        };
        let captured = Captured::default();
        run(std::io::BufReader::new(input), captured.clone());
        String::from_utf8(captured.0.lock().unwrap().clone()).unwrap()
    }

    fn timed(input: &str) -> (String, Duration) {
        let started = Instant::now();
        let reply = respond(input);
        (reply, started.elapsed())
    }

    #[test]
    fn handshake() {
        let reply = respond("uci\n");
        assert!(reply.starts_with("id name Stargazer"));
        assert!(reply.ends_with("uciok\n"));
    }

    #[test]
    fn the_hash_option_is_offered_and_understood() {
        assert!(
            respond("uci\n").contains("option name Hash type spin default 16 min 1 max 1024\n")
        );
        let hash = |line| parse_option(line);
        assert_eq!(
            hash("setoption name Hash value 64"),
            Some(EngineOption::Hash(64))
        );
        assert_eq!(
            hash("setoption name hash value 0"),
            Some(EngineOption::Hash(1))
        );
        assert_eq!(
            hash("setoption name Hash value 99999"),
            Some(EngineOption::Hash(1024))
        );
        assert_eq!(hash("setoption name Hash value lots"), None);
        assert_eq!(hash("setoption name Ponder value true"), None);
        // A search still works after resizing and after a new game.
        let reply = respond("setoption name Hash value 1\nucinewgame\ngo depth 3\n");
        assert!(reply.contains("bestmove "), "{reply}");
        assert!(respond("setoption name Ponder value true\n").starts_with("info string"));
    }

    #[test]
    fn the_move_overhead_option_is_offered_and_understood() {
        assert!(
            respond("uci\n")
                .contains("option name Move Overhead type spin default 50 min 0 max 5000\n")
        );
        assert_eq!(
            parse_option("setoption name Move Overhead value 200"),
            Some(EngineOption::MoveOverhead(200))
        );
        assert_eq!(
            parse_option("setoption name move overhead value 99999"),
            Some(EngineOption::MoveOverhead(5000))
        );
        assert_eq!(parse_option("setoption name Move Overhead value -1"), None);
        assert_eq!(parse_option("setoption name Move Overhead"), None);
        // It's what's kept back: a 1-second move time with 300 ms kept
        // back leaves 700 ms.
        let limits = parse_go("go movetime 1000");
        assert_eq!(
            time_budget(&limits, Color::White, 24, 300),
            Some(TimeBudget {
                soft: 700,
                hard: 700
            })
        );
        // And the loop passes the option on: with 900 of a 1000 ms move
        // time kept back, the answer comes in about 100 ms, not 950.
        let (reply, took) =
            timed("setoption name Move Overhead value 900\nposition startpos\ngo movetime 1000\n");
        assert!(reply.contains("bestmove "), "{reply}");
        assert!(took < Duration::from_millis(500), "{took:?}");
    }

    #[test]
    fn a_lone_legal_move_is_played_at_once() {
        // White's king on h1 is checked by the rook on a1 and can only go
        // to h2: g1 is on the rook's line, and the bishop on g2 is guarded
        // by the pawn on f3.
        let fen = "position fen 7k/8/8/8/8/5p2/6b1/r6K w - - 0 1";
        let (reply, took) = timed(&format!("{fen}\ngo wtime 60000 btime 60000\n"));
        assert_eq!(reply, "bestmove h1h2\n");
        assert!(took < Duration::from_millis(500), "{took:?}");
        // With a depth asked for, it still searches and reports.
        let reply = respond(&format!("{fen}\ngo depth 3\n"));
        assert!(reply.contains("info depth 3"), "{reply}");
        assert!(reply.ends_with("bestmove h1h2\n"), "{reply}");
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

    #[test]
    fn go_with_a_clock_moves_in_time() {
        // One second each: the target is 1000 / 20 = 50 ms, the ceiling 150 ms.
        let (reply, took) = timed("go wtime 1000 btime 1000\n");
        assert!(reply.contains("bestmove "), "{reply}");
        assert!(took < Duration::from_millis(1000), "took {took:?}");
    }

    #[test]
    fn go_movetime_is_respected() {
        let (reply, took) = timed("go movetime 200\n");
        assert!(reply.contains("bestmove "), "{reply}");
        assert!(took < Duration::from_millis(1000), "took {took:?}");
    }

    #[test]
    fn go_infinite_waits_for_stop() {
        // Kings only: every depth is instant, so the search runs out of
        // depths long before `stop`, but must still wait for it.
        let reply = respond_paced(&[
            (0, "position fen 4k3/8/8/8/8/8/8/4K3 w - - 0 1"),
            (0, "go infinite"),
            (300, "isready"),
            (0, "stop"),
        ]);
        let ready = reply
            .find("readyok")
            .expect("isready answered while thinking");
        let best = reply.find("bestmove").expect("bestmove after stop");
        assert!(ready < best, "{reply}");
        assert_eq!(reply.matches("bestmove").count(), 1);
    }

    #[test]
    fn stop_ends_a_long_search_quickly() {
        let (reply, took) = timed("go infinite\nstop\n");
        assert!(
            reply.ends_with('\n') && reply.contains("bestmove "),
            "{reply}"
        );
        assert!(took < Duration::from_secs(2), "took {took:?}");
    }

    #[test]
    fn reading_the_go_command() {
        let limits = parse_go("go wtime 60000 btime 55000 winc 1000 binc 2000 movestogo 30");
        assert_eq!(limits.time, [Some(60000), Some(55000)]);
        assert_eq!(limits.increment, [1000, 2000]);
        assert_eq!(limits.moves_to_go, Some(30));
        assert!(!limits.infinite && limits.depth.is_none());
        // A clock that ran out is sometimes sent as a negative number.
        assert_eq!(parse_go("go wtime -20 btime 100").time[0], Some(0));
    }

    #[test]
    fn sudden_death_expects_fewer_moves_as_pieces_come_off() {
        // Every piece on (phase 24) is capped at 45.
        assert_eq!(sudden_death_moves_left(24), 45);
        assert_eq!(sudden_death_moves_left(20), 45);
        // A queen and a rook each (12).
        assert_eq!(sudden_death_moves_left(12), 37);
        // Knight against bishop (2).
        assert_eq!(sudden_death_moves_left(2), 27);
        // Kings and pawns only.
        assert_eq!(sudden_death_moves_left(0), 25);
    }

    #[test]
    fn the_time_budget() {
        let clock = |wtime, btime, winc, binc| GoLimits {
            time: [Some(wtime), Some(btime)],
            increment: [winc, binc],
            ..GoLimits::default()
        };
        // A minute with an increment: a twentieth plus most of the
        // increment, three times that as the ceiling.
        assert_eq!(
            time_budget(&clock(60_000, 1, 1_000, 0), Color::White, 24, 50),
            Some(TimeBudget {
                soft: 3_750,
                hard: 11_250
            })
        );
        // A minute, no increment (sudden death), every piece on the board:
        // shared over 45 moves, ceiling three times that (under a tenth of
        // the clock).
        assert_eq!(
            time_budget(&clock(60_000, 1, 0, 0), Color::White, 24, 50),
            Some(TimeBudget {
                soft: 1_333,
                hard: 3_999
            })
        );
        // Only kings and pawns left: shared over 25 moves, and the ceiling
        // is held to a tenth of the clock.
        assert_eq!(
            time_budget(&clock(60_000, 1, 0, 0), Color::White, 0, 50),
            Some(TimeBudget {
                soft: 2_400,
                hard: 6_000
            })
        );
        // A time control coming (moves to go): shared out over those moves,
        // increment or not.
        let mut control = clock(60_000, 1, 0, 0);
        control.moves_to_go = Some(9);
        assert_eq!(
            time_budget(&control, Color::White, 24, 50).map(|b| b.soft),
            Some(6_000)
        );
        // The side to move's clock is the one used.
        assert_eq!(
            time_budget(&clock(1, 60_000, 0, 2_000), Color::Black, 24, 50),
            Some(TimeBudget {
                soft: 4_500,
                hard: 13_500
            })
        );
        // Almost out of time: never plan more than what's left, minus the margin.
        let low = time_budget(&clock(1_000, 1_000, 3_000, 3_000), Color::White, 24, 50).unwrap();
        assert_eq!(low.hard, 950);
        assert!(low.soft <= low.hard);
        // When to stop: inside a depth at the ceiling, between depths past
        // 60% of the target.
        let budget = TimeBudget {
            soft: 1_000,
            hard: 3_000,
        };
        assert!(budget.worth_another_depth(599, 100) && !budget.worth_another_depth(600, 100));
        // Stretched for the position, but never past the ceiling.
        assert!(budget.worth_another_depth(1_199, 200) && !budget.worth_another_depth(1_200, 200));
        assert!(
            budget.worth_another_depth(1_799, 1_000) && !budget.worth_another_depth(1_800, 1_000)
        );
        assert!(!budget.out_of_time(2_999) && budget.out_of_time(3_000));
        // No clock: no time limit.
        assert_eq!(
            time_budget(&GoLimits::default(), Color::White, 24, 50),
            None
        );
    }

    #[test]
    fn a_falling_score_gets_twice_the_time() {
        let mut scores = ScoreWatch::default();
        // The first depth: nothing to compare with yet.
        assert_eq!(scores.after_depth(40), 100);
        // Down by exactly a third of a pawn: not yet a problem.
        assert_eq!(scores.after_depth(10), 100);
        // Down by more: twice the target.
        assert_eq!(scores.after_depth(-21), 200);
        // Steady again: as planned.
        assert_eq!(scores.after_depth(-21), 100);
        // A rising score changes nothing.
        assert_eq!(scores.after_depth(300), 100);
    }

    #[test]
    fn bench_prints_nodes_and_speed() {
        let reply = respond("bench 2\n");
        let words: Vec<&str> = reply.split_whitespace().collect();
        assert_eq!(words[0], "Bench:");
        assert!(words[1].parse::<u64>().unwrap() > 0, "{reply}");
        assert_eq!((words[2], words[4]), ("nodes", "nps"), "{reply}");
    }
}
