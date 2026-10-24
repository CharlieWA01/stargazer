//! Search: looking ahead to choose a move (DESIGN.md section 3).
//!
//! The idea in chess terms: "if I play this, what's their best reply, and
//! my best answer to that, and so on", a few moves deep, then judge the
//! final positions with `evaluate`. Both sides are assumed to play their
//! best, so each side picks the move that's worst for the other.
//!
//! The pieces:
//! - **Negamax with alpha-beta:** stop looking at a move as soon as it's
//!   clear the opponent has a refutation that makes it worse than a move
//!   already found. Same answer as looking at everything, far fewer positions.
//! - **Iterative deepening:** search 1 move deep, then 2, then 3, until
//!   told to stop. Each finished depth gives a ready answer, and its best
//!   line is tried first next time, which makes alpha-beta cut sooner.
//! - **Quiescence search:** at the end of the line, keep playing out
//!   captures until the position is quiet, so the engine never judges a
//!   position halfway through an exchange.
//! - **Principal variation search (PVS):** the first move tried is
//!   usually the best, so it's searched properly; each later move only
//!   has to answer "is this better than the first?", which a quick
//!   search with a zero-width window settles. Only a "yes" is searched
//!   again properly, to find out by how much.
//! - **Aspiration windows:** each depth's score is usually close to the
//!   last one's, so from depth 5 the root is searched only for scores
//!   within a quarter of a pawn of it, which cuts off more. If the real
//!   score is outside, the window is widened and that depth searched
//!   again.
//! - **Captures first, best first:** take the most valuable piece with the
//!   least valuable attacker first ("MVV-LVA"). Good moves first means
//!   more cut-offs.
//! - **Killer moves:** a quiet move (not a capture) that refuted one
//!   position will often refute its neighbours at the same depth too:
//!   if ...Qh4 punished a3, it probably punishes h3 as well. The last two
//!   such moves at each depth are tried right after the captures.
//! - **History:** every quiet move has a running score: up when it refutes
//!   a position, down when it was tried first and failed. The other quiet
//!   moves are tried highest score first. It's a memory of "moves that
//!   tend to be good in this game", like Rd1 when the d-file matters.
//! - **Transposition table:** remember each position's result (see `tt`),
//!   so a position reached again by another move order isn't searched
//!   again, and its best move is tried first when it is.
//! - **Results:** checkmate (quicker mates score higher), stalemate,
//!   repetition, the 50-move rule and insufficient material.
//!
//! The search has no clock. It's handed a `should_stop` question, which it
//! asks every few thousand positions, and reports after each depth. The
//! caller (the UCI layer, or the browser page later) owns the time.

use crate::board::{Color, PieceKind, Position};
use crate::eval::evaluate;
use crate::movegen::{Move, legal_moves};
use crate::tt::{Bound, Table};

/// The score for checkmating. A mate found `ply` half-moves from now scores
/// `MATE - ply`, so the engine prefers the quickest mate (and, when losing,
/// the slowest).
pub const MATE: i32 = 30_000;
/// Larger than any real score.
const INFINITY: i32 = 32_000;
/// Deepest the search (including captures at the end) ever goes, in half-moves.
pub const MAX_PLY: usize = 128;
/// How often, in positions, to ask whether to stop.
const CHECK_EVERY: u64 = 2048;
/// Aspiration windows start at this depth (shallower scores jump about).
const ASPIRATION_DEPTH: u32 = 5;
/// The first window: this many centipawns either side of last depth's score.
const ASPIRATION_WINDOW: i32 = 25;
/// Past this, a missed window gives up and searches everything.
const ASPIRATION_LIMIT: i32 = 400;

/// The two most recent quiet moves that refuted a position at one ply,
/// newest first.
type Killers = [Option<Move>; 2];

/// The furthest a history score goes either way.
const HISTORY_MAX: i32 = 16_384;

/// History scores for quiet moves, by side to move, from-square and
/// to-square. Kept by the caller from move to move, like the table.
#[derive(Clone)]
pub struct History {
    scores: Box<[[[i32; 64]; 64]; 2]>,
}

impl Default for History {
    fn default() -> History {
        History::new()
    }
}

impl History {
    /// No history: every quiet move scores 0.
    pub fn new() -> History {
        History {
            scores: Box::new([[[0; 64]; 64]; 2]),
        }
    }

    /// Forget everything (a new game).
    pub fn clear(&mut self) {
        *self = History::new();
    }

    /// Halve every score, between moves of a game: old patterns fade but
    /// aren't lost.
    pub fn age(&mut self) {
        for score in self.scores.iter_mut().flatten().flatten() {
            *score /= 2;
        }
    }

    fn score(&self, side: Color, mv: Move) -> i32 {
        self.scores[side.index()][mv.from.index()][mv.to.index()]
    }

    /// Move `mv`'s score by `change` (up for a refutation, down for a
    /// failure). The nearer a score is to the limit, the less it moves,
    /// so it can never pass `HISTORY_MAX` either way.
    fn adjust(&mut self, side: Color, mv: Move, change: i32) {
        let score = &mut self.scores[side.index()][mv.from.index()][mv.to.index()];
        *score += change - *score * change.abs() / HISTORY_MAX;
    }
}

/// What a finished depth found, for `info` lines.
#[derive(Clone, Debug)]
pub struct Report {
    pub depth: u32,
    /// Centipawns for the side to move, or near `MATE` for a forced mate.
    pub score: i32,
    /// Positions looked at so far, all depths together.
    pub nodes: u64,
    /// The best line: our move, their best reply, and so on.
    pub pv: Vec<Move>,
}

/// If `score` means a forced mate, in how many full moves: positive when we
/// mate, negative when we get mated. `None` for an ordinary score.
pub fn mate_in(score: i32) -> Option<i32> {
    let plies = MATE - score.abs();
    if plies > MAX_PLY as i32 {
        return None;
    }
    let moves = (plies + 1) / 2;
    Some(if score > 0 { moves } else { -moves })
}

/// Find the best move.
///
/// - `game`: every position in the game so far, the current one last (for
///   the repetition rule).
/// - `max_depth`: the deepest iteration to run, in half-moves.
/// - `table`: the transposition table, kept by the caller from move to
///   move (what it learned last move often still applies).
/// - `history`: the quiet moves' history scores, also kept from move to
///   move; halved at the start of each search.
/// - `should_stop`: asked now and then during depth 2 and later; once it
///   answers yes, the search stops. Moves already finished in the
///   unfinished depth still count: if one beat last depth's best move, it
///   is played. Depth 1 always finishes, so there's always a move.
/// - `on_depth`: called after each finished depth; return false to stop
///   there (the caller's time check between depths).
///
/// Returns `None` only when there's no legal move (checkmate or stalemate).
pub fn search(
    game: &[Position],
    max_depth: u32,
    table: &mut Table,
    history: &mut History,
    should_stop: &mut dyn FnMut() -> bool,
    on_depth: &mut dyn FnMut(&Report) -> bool,
) -> Option<Move> {
    let all = Features {
        pvs: true,
        aspiration: true,
    };
    search_with(game, max_depth, table, history, should_stop, on_depth, all)
}

/// Search tricks that change the work but never the answer, so tests can
/// switch them off and check that the scores stay the same. Always all on
/// outside tests.
#[derive(Clone, Copy)]
struct Features {
    pvs: bool,
    aspiration: bool,
}

/// `search`, with some features switched off (in tests only).
fn search_with(
    game: &[Position],
    max_depth: u32,
    table: &mut Table,
    history: &mut History,
    should_stop: &mut dyn FnMut() -> bool,
    on_depth: &mut dyn FnMut(&Report) -> bool,
    features: Features,
) -> Option<Move> {
    let (root, earlier) = game.split_last()?;
    let first_legal = *legal_moves(root).first()?;
    history.age();

    let mut searcher = Searcher {
        path: earlier.to_vec(),
        nodes: 0,
        may_stop: false,
        stopped: false,
        should_stop,
        table,
        killers: [[None; 2]; MAX_PLY + 1],
        history,
        quiets_tried: vec![Vec::new(); MAX_PLY + 1],
        pvs: features.pvs,
    };
    let mut best_move = first_legal;
    let mut previous_score = 0;
    let mut previous_pv = Vec::new();

    for depth in 1..=max_depth.min(MAX_PLY as u32 / 2) {
        searcher.may_stop = depth > 1;
        let mut pv = Vec::new();
        let score = if features.aspiration
            && depth >= ASPIRATION_DEPTH
            && mate_in(previous_score).is_none()
        {
            searcher.aspiration(root, depth, previous_score, &mut pv, &previous_pv)
        } else {
            searcher.negamax(root, depth, 0, -INFINITY, INFINITY, &mut pv, &previous_pv)
        };
        if searcher.stopped {
            // Out of time partway through this depth. Last depth's best
            // move was searched first, so a best line here means that move
            // was finished at this depth and either it held up or another
            // move beat it: either way, a better-informed choice than last
            // depth's. Only an unfinished move is thrown away.
            if let Some(&mv) = pv.first() {
                best_move = mv;
            }
            break;
        }
        if let Some(&mv) = pv.first() {
            best_move = mv;
        }
        previous_pv = pv.clone();
        previous_score = score;
        let report = Report {
            depth,
            score,
            nodes: searcher.nodes,
            pv,
        };
        if !on_depth(&report) {
            break;
        }
    }
    Some(best_move)
}

struct Searcher<'a> {
    /// Positions before the one being searched: the game, then the moves
    /// along the current line. Used to spot repetitions.
    path: Vec<Position>,
    nodes: u64,
    /// False during depth 1, so there's always a finished answer.
    may_stop: bool,
    stopped: bool,
    should_stop: &'a mut dyn FnMut() -> bool,
    table: &'a mut Table,
    /// Killer moves for each ply, kept across depths of one search.
    killers: [Killers; MAX_PLY + 1],
    history: &'a mut History,
    /// For each ply, a reusable list of the quiet moves tried there.
    quiets_tried: Vec<Vec<Move>>,
    /// Principal variation search on (always, outside tests).
    pvs: bool,
}

impl Searcher<'_> {
    /// Count one more position, and every few thousand ask whether to stop.
    fn visit(&mut self) -> bool {
        self.nodes += 1;
        if self.may_stop && self.nodes.is_multiple_of(CHECK_EVERY) && (self.should_stop)() {
            self.stopped = true;
        }
        self.stopped
    }

    /// Search the root at `depth` expecting a score near `guess` (last
    /// depth's). Start with a window of a quarter of a pawn either side;
    /// each time the score lands outside, double the window on that side
    /// and search again, until it gives up and searches everything.
    fn aspiration(
        &mut self,
        root: &Position,
        depth: u32,
        guess: i32,
        pv: &mut Vec<Move>,
        hint: &[Move],
    ) -> i32 {
        let mut window = ASPIRATION_WINDOW;
        let (mut alpha, mut beta) = (guess - window, guess + window);
        // A move that beat the window is better than expected: keep it in
        // case time runs out while finding out by how much.
        let mut proven = None;
        loop {
            let score = self.negamax(root, depth, 0, alpha, beta, pv, hint);
            if self.stopped {
                if pv.is_empty()
                    && let Some(mv) = proven
                {
                    pv.push(mv);
                }
                return score;
            }
            if score > alpha && score < beta {
                return score;
            }
            window *= 2;
            if score <= alpha {
                // Worse than expected: look lower.
                alpha = (score - window).max(-INFINITY);
            } else {
                // Better than expected: look higher.
                proven = pv.first().copied();
                beta = (score + window).min(INFINITY);
            }
            if window > ASPIRATION_LIMIT {
                (alpha, beta) = (-INFINITY, INFINITY);
            }
        }
    }

    /// The score of `position` for the side to move, looking `depth`
    /// half-moves ahead. `alpha` is the score we're already sure of
    /// elsewhere; `beta` is the most the opponent will allow. Fills `pv`
    /// with the best line. `hint` is last iteration's best line from here,
    /// tried first.
    #[allow(clippy::too_many_arguments)]
    fn negamax(
        &mut self,
        position: &Position,
        depth: u32,
        ply: usize,
        mut alpha: i32,
        beta: i32,
        pv: &mut Vec<Move>,
        hint: &[Move],
    ) -> i32 {
        pv.clear();
        if self.visit() {
            return 0;
        }
        // Draws that don't depend on the moves (not at the root: there we
        // still have to pick a move).
        if ply > 0 && (position.insufficient_material() || self.is_repetition(position)) {
            return 0;
        }

        let moves = legal_moves(position);
        if moves.is_empty() {
            return if position.in_check(position.side_to_move) {
                -(MATE - ply as i32) // checkmated
            } else {
                0 // stalemate
            };
        }
        // 50 moves each without a capture or pawn move (checked after
        // checkmate, which takes priority).
        if ply > 0 && position.halfmove_clock >= 100 {
            return 0;
        }
        if depth == 0 || ply >= MAX_PLY {
            return self.quiescence(position, ply, alpha, beta);
        }

        // Seen this position before? If it was searched at least this deep
        // and its score settles the question here, use it (not at the
        // root, which must always come back with a move).
        let remembered = self.table.probe(position.hash(), ply);
        if let Some(found) = remembered
            && ply > 0
            && found.depth >= depth
        {
            let settled = match found.bound {
                Bound::Exact => true,
                Bound::Lower => found.score >= beta,
                Bound::Upper => found.score <= alpha,
            };
            if settled {
                return found.score;
            }
        }

        // Try last iteration's best move first, else the remembered one.
        let first = hint
            .first()
            .copied()
            .or(remembered.and_then(|f| f.best_move));
        let moves = ordered(
            position,
            moves,
            first,
            self.killers[ply],
            Some(&*self.history),
        );
        let alpha_at_start = alpha;
        let mut best = -INFINITY;
        let mut best_move = None;
        let mut line = Vec::new();
        // Quiet moves tried here that didn't refute the position (a list
        // kept per ply and reused, so no new memory is needed each time).
        let mut quiets_tried = std::mem::take(&mut self.quiets_tried[ply]);
        quiets_tried.clear();
        self.path.push(*position);
        for mv in moves {
            // Follow last iteration's best line only down its own first move.
            let child_hint = match hint.split_first() {
                Some((&first, rest)) if first == mv => rest,
                _ => &[],
            };
            let next = position.make_move(mv);
            let first_move = best == -INFINITY;
            let mut score;
            if first_move || !self.pvs {
                // The first move (usually the best): search it properly.
                score = -self.negamax(
                    &next,
                    depth - 1,
                    ply + 1,
                    -beta,
                    -alpha,
                    &mut line,
                    child_hint,
                );
            } else {
                // A later move: only ask whether it beats the best so far,
                // with a window too narrow for any score to fit inside.
                score = -self.negamax(
                    &next,
                    depth - 1,
                    ply + 1,
                    -alpha - 1,
                    -alpha,
                    &mut line,
                    child_hint,
                );
                // It does beat it, and might matter here: find out by how
                // much with a proper search.
                if score > alpha && score < beta && !self.stopped {
                    score = -self.negamax(
                        &next,
                        depth - 1,
                        ply + 1,
                        -beta,
                        -alpha,
                        &mut line,
                        child_hint,
                    );
                    if self.stopped && ply == 0 {
                        // Out of time before finding out by how much. At
                        // the root that's still enough: this move is
                        // proven better than the best so far, so it's the
                        // one to play.
                        pv.clear();
                        pv.push(mv);
                    }
                }
            }
            if self.stopped {
                break;
            }
            if score > best {
                best = score;
                best_move = Some(mv);
            }
            if score > alpha {
                // A new best move: remember the line it leads to.
                alpha = score;
                pv.clear();
                pv.push(mv);
                pv.extend_from_slice(&line);
            }
            if alpha >= beta {
                // The opponent won't allow this line; no need to look at
                // our other moves here. Remember the refutation, in case
                // it refutes the neighbouring positions too.
                self.remember_killer(position, ply, mv);
                self.update_history(position, depth, mv, &quiets_tried);
                break;
            }
            if is_quiet(position, mv) {
                quiets_tried.push(mv);
            }
        }
        self.path.pop();
        self.quiets_tried[ply] = quiets_tried;
        if !self.stopped {
            let bound = if best >= beta {
                Bound::Lower
            } else if best > alpha_at_start {
                Bound::Exact
            } else {
                Bound::Upper
            };
            self.table
                .store(position.hash(), ply, depth, best, bound, best_move);
        }
        best
    }

    /// At the end of the line, play out captures until the position is
    /// quiet. The side to move may also "stand pat": decline every capture
    /// and take the position's evaluation as it is. In check there's no
    /// standing pat: every way out of check is tried.
    fn quiescence(&mut self, position: &Position, ply: usize, mut alpha: i32, beta: i32) -> i32 {
        if self.visit() {
            return 0;
        }
        let in_check = position.in_check(position.side_to_move);
        let mut moves = legal_moves(position);
        if moves.is_empty() {
            return if in_check { -(MATE - ply as i32) } else { 0 };
        }
        if ply >= MAX_PLY {
            return evaluate(position);
        }

        let mut best = -INFINITY;
        if !in_check {
            let stand_pat = evaluate(position);
            if stand_pat >= beta {
                return stand_pat;
            }
            alpha = alpha.max(stand_pat);
            best = stand_pat;
            moves.retain(|&mv| is_capture(position, mv) || mv.promotion.is_some());
        }

        for mv in ordered(position, moves, None, [None; 2], None) {
            let next = position.make_move(mv);
            let score = -self.quiescence(&next, ply + 1, -beta, -alpha);
            if self.stopped {
                return 0;
            }
            best = best.max(score);
            alpha = alpha.max(score);
            if alpha >= beta {
                break;
            }
        }
        best
    }

    /// `mv` refuted `position`: if it's a quiet move, make it this ply's
    /// newest killer. Captures and promotions are left out; they're
    /// already tried early.
    fn remember_killer(&mut self, position: &Position, ply: usize, mv: Move) {
        if !is_quiet(position, mv) {
            return;
        }
        let killers = &mut self.killers[ply];
        if killers[0] != Some(mv) {
            killers[1] = killers[0];
            killers[0] = Some(mv);
        }
    }

    /// `mv` refuted `position` with `depth` half-moves to go: if it's
    /// quiet, raise its history score, and lower the scores of the quiet
    /// moves tried before it that failed. Deeper searches count for more
    /// (depth × depth), since their verdicts are more reliable.
    fn update_history(&mut self, position: &Position, depth: u32, mv: Move, failed: &[Move]) {
        if !is_quiet(position, mv) {
            return;
        }
        let bonus = (depth * depth).min(HISTORY_MAX as u32) as i32;
        let side = position.side_to_move;
        self.history.adjust(side, mv, bonus);
        for &other in failed {
            self.history.adjust(side, other, -bonus);
        }
    }

    /// Has this position appeared before, since the last capture or pawn
    /// move? Once is enough to call it a draw inside the search: if a
    /// position can be repeated once, it can be repeated again.
    fn is_repetition(&self, position: &Position) -> bool {
        // Only positions with the same side to move can match: every second one.
        let lookback = position.halfmove_clock as usize;
        self.path
            .iter()
            .rev()
            .take(lookback)
            .skip(1)
            .step_by(2)
            .any(|earlier| earlier.repeats(position))
    }
}

/// A quiet move: neither a capture nor a promotion.
fn is_quiet(position: &Position, mv: Move) -> bool {
    !is_capture(position, mv) && mv.promotion.is_none()
}

/// Does this move capture something (en passant included)?
fn is_capture(position: &Position, mv: Move) -> bool {
    position.piece_on(mv.to).is_some() || is_en_passant(position, mv)
}

fn is_en_passant(position: &Position, mv: Move) -> bool {
    Some(mv.to) == position.en_passant
        && position
            .piece_on(mv.from)
            .is_some_and(|piece| piece.kind == PieceKind::Pawn)
}

/// Rough value of each piece kind for ordering captures (pawn, knight,
/// bishop, rook, queen, king).
fn order_value(kind: PieceKind) -> i32 {
    match kind {
        PieceKind::Pawn => 1,
        PieceKind::Knight | PieceKind::Bishop => 3,
        PieceKind::Rook => 5,
        PieceKind::Queen => 9,
        PieceKind::King => 10,
    }
}

/// Put the moves most likely to be best first: last iteration's best move,
/// then captures (biggest victim first, cheapest attacker first among
/// equal victims) and promotions, then the killer moves (newest first),
/// then the other quiet moves, highest history score first.
fn ordered(
    position: &Position,
    mut moves: Vec<Move>,
    best_first: Option<Move>,
    killers: Killers,
    history: Option<&History>,
) -> Vec<Move> {
    // Each group's priorities sit far above the next group's.
    const FIRST: i32 = 3_000_000;
    const CAPTURES: i32 = 2_000_000;
    const KILLERS: i32 = 1_000_000;
    let priority = |mv: &Move| {
        if Some(*mv) == best_first {
            return FIRST;
        }
        let victim = if is_en_passant(position, *mv) {
            Some(PieceKind::Pawn)
        } else {
            position.piece_on(mv.to).map(|piece| piece.kind)
        };
        if victim.is_some() || mv.promotion.is_some() {
            let mut priority = CAPTURES;
            if let Some(victim) = victim {
                let attacker = position
                    .piece_on(mv.from)
                    .map_or(PieceKind::Pawn, |p| p.kind);
                priority += 10 * order_value(victim) - order_value(attacker) + 20;
            }
            if let Some(kind) = mv.promotion {
                priority += 10 * order_value(kind);
            }
            return priority;
        }
        // A quiet move here. (A killer is quiet where it was found, but
        // here the same move may capture; then it's ranked as a capture.)
        if Some(*mv) == killers[0] {
            KILLERS + 1
        } else if Some(*mv) == killers[1] {
            KILLERS
        } else {
            history.map_or(0, |h| h.score(position.side_to_move, *mv))
        }
    };
    // Highest priority first; ties keep move generation's order.
    moves.sort_by_key(|mv| -priority(mv));
    moves
}

#[cfg(test)]
mod tests {
    use super::*;

    fn position(fen: &str) -> Position {
        Position::from_fen(fen).unwrap()
    }

    /// Search `fen` to `depth`; return the best move and the last report.
    fn search_fen(fen: &str, depth: u32) -> (Move, Report) {
        let mut last = None;
        let best = search(
            &[position(fen)],
            depth,
            &mut Table::new(1),
            &mut History::new(),
            &mut || false,
            &mut |report| {
                last = Some(report.clone());
                true
            },
        )
        .unwrap();
        (best, last.unwrap())
    }

    #[test]
    fn mate_in_one() {
        // Back-rank mate: the rook goes to a8.
        let (best, report) = search_fen("6k1/5ppp/8/8/8/8/8/R5K1 w - - 0 1", 2);
        assert_eq!(best.to_string(), "a1a8");
        assert_eq!(mate_in(report.score), Some(1));
    }

    #[test]
    fn mate_in_two_with_two_rooks() {
        // The "ladder": one rook cuts the king off on the back rank, the
        // other mates. No mate in one exists here.
        let fen = "7k/8/8/8/8/8/R7/1R4K1 w - - 0 1";
        let (_, report) = search_fen(fen, 3);
        assert_eq!(mate_in(report.score), Some(2), "{report:?}");
        // Playing out the best line really does end in checkmate.
        let mut board = position(fen);
        for &mv in &report.pv {
            board = board.make_move(mv);
        }
        assert_eq!(report.pv.len(), 3);
        assert!(legal_moves(&board).is_empty() && board.in_check(board.side_to_move));
    }

    #[test]
    fn being_mated_is_reported_as_negative() {
        // Black to move: the rook on a7 holds the king to the back rank,
        // Kg8 is forced, then Rb8 is mate.
        let (_, report) = search_fen("7k/R7/8/8/8/8/8/1R4K1 b - - 0 1", 3);
        assert_eq!(mate_in(report.score), Some(-1), "{report:?}");
    }

    #[test]
    fn prefers_mate_to_stalemate() {
        // Qc7 would stalemate the king on a8; Qc8 is mate.
        let (best, report) = search_fen("k7/8/1K6/8/8/8/8/2Q5 w - - 0 1", 2);
        assert_eq!(best.to_string(), "c1c8");
        assert_eq!(mate_in(report.score), Some(1));
    }

    #[test]
    fn takes_a_free_queen() {
        let (best, _) = search_fen("4k3/8/8/3q4/8/8/8/3RK3 w - - 0 1", 2);
        assert_eq!(best.to_string(), "d1d5");
    }

    #[test]
    fn does_not_grab_a_defended_pawn_with_the_queen() {
        // Qxd5 wins a pawn but loses the queen to cxd5. Even at depth 1,
        // quiescence plays out the recapture and sees it.
        let (best, report) = search_fen("4k3/8/2p5/3p4/8/8/8/3QK3 w - - 0 1", 1);
        assert_ne!(best.to_string(), "d1d5", "{report:?}");
        assert!(report.score > 0);
    }

    #[test]
    fn rescues_an_attacked_queen() {
        // The pawn on c5 attacks the queen on d4. Moving the king (or
        // anything else) would lose her; the score shows she's kept.
        let (_, report) = search_fen("4k3/8/8/2p5/3Q4/8/8/4K3 w - - 0 1", 2);
        assert!(report.score > 700, "{report:?}");
    }

    #[test]
    fn the_fifty_move_rule_turns_a_win_into_a_draw() {
        // A rook up, but every move reaches 100 half-moves with no capture
        // or pawn move, and none of them is checkmate.
        let (_, report) = search_fen("4k3/8/8/8/8/8/8/R3K3 w - - 99 80", 2);
        assert_eq!(report.score, 0);
        let (_, report) = search_fen("4k3/8/8/8/8/8/8/R3K3 w - - 0 80", 2);
        assert!(report.score > 400);
    }

    #[test]
    fn insufficient_material_is_a_draw() {
        // White can take the last pawn, leaving king and knight vs king:
        // still a draw, so the score is 0 whichever way.
        let (_, report) = search_fen("4k3/8/8/8/8/8/3p4/2N1K3 w - - 0 1", 2);
        assert_eq!(report.score, 0, "{report:?}");
    }

    /// A searcher that has `path` behind it, to test the repetition rule.
    fn is_repetition_after(path: &[Position], position: &Position) -> bool {
        let mut never = || false;
        let mut table = Table::new(1);
        let mut history = History::new();
        let searcher = Searcher {
            path: path.to_vec(),
            nodes: 0,
            may_stop: false,
            stopped: false,
            should_stop: &mut never,
            table: &mut table,
            killers: [[None; 2]; MAX_PLY + 1],
            history: &mut history,
            quiets_tried: vec![Vec::new(); MAX_PLY + 1],
            pvs: true,
        };
        searcher.is_repetition(position)
    }

    #[test]
    fn knights_out_and_back_repeat_the_position() {
        // 1. Nf3 Nf6 2. Ng1 Ng8 brings back the start position.
        let mut game = vec![Position::startpos()];
        for text in ["g1f3", "g8f6", "f3g1", "f6g8"] {
            let current = *game.last().unwrap();
            let mv = legal_moves(&current)
                .into_iter()
                .find(|m| m.to_string() == text)
                .unwrap();
            game.push(current.make_move(mv));
        }
        let (now, before) = game.split_last().unwrap();
        assert!(is_repetition_after(before, now));
        // Without the start position in the history, nothing repeats.
        assert!(!is_repetition_after(&before[1..], now));
    }

    #[test]
    fn stops_when_asked_but_always_finishes_depth_one() {
        let mut asked = 0;
        let mut depths = Vec::new();
        let best = search(
            &[Position::startpos()],
            20,
            &mut Table::new(1),
            &mut History::new(),
            &mut || {
                asked += 1;
                true
            },
            &mut |report| {
                depths.push(report.depth);
                true
            },
        );
        assert!(best.is_some());
        assert!(asked > 0);
        assert_eq!(depths[0], 1);
        assert!(depths.len() < 20);
    }

    /// Search to `max_depth`, stopping at the `stop_at`-th time the search
    /// asks whether to stop (never, if `None`). Returns the move played,
    /// and for each finished depth: how many times it had asked by then,
    /// and that depth's best move.
    fn search_counting(
        fen: &str,
        max_depth: u32,
        stop_at: Option<u64>,
    ) -> (Move, Vec<(u64, Move)>) {
        let asked = std::cell::Cell::new(0);
        let mut depths = Vec::new();
        let best = search(
            &[position(fen)],
            max_depth,
            &mut Table::new(1),
            &mut History::new(),
            &mut || {
                asked.set(asked.get() + 1);
                stop_at.is_some_and(|n| asked.get() >= n)
            },
            &mut |report| {
                depths.push((asked.get(), report.pv[0]));
                true
            },
        )
        .unwrap();
        (best, depths)
    }

    #[test]
    fn a_better_move_found_in_an_unfinished_depth_is_played() {
        // Here depth 4 likes d2-d4 and depth 5 changes its mind to c4-c5.
        let fen = "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1";
        let (_, depths) = search_counting(fen, 5, None);
        let (asked_after_4, best_at_4) = depths[3];
        let (asked_after_5, best_at_5) = depths[4];
        assert_ne!(best_at_4, best_at_5);

        // Out of time at the last check of depth 5, just before it would
        // have finished: c4-c5 has already beaten d2-d4, so it's played
        // even though depth 5 never completed.
        let (best, finished) = search_counting(fen, 5, Some(asked_after_5));
        assert_eq!(finished.len(), 4);
        assert_eq!(best, best_at_5);

        // Out of time at the first check of depth 5, while d2-d4 (tried
        // first) is still being searched: nothing at depth 5 is finished,
        // so depth 4's move stands.
        let (best, finished) = search_counting(fen, 5, Some(asked_after_4 + 1));
        assert_eq!(finished.len(), 4);
        assert_eq!(best, best_at_4);
    }

    #[test]
    fn the_table_saves_work_and_remembers_between_searches() {
        let kiwipete = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
        let nodes = |table: &mut Table| {
            let mut last = 0;
            search(
                &[position(kiwipete)],
                4,
                table,
                &mut History::new(),
                &mut || false,
                &mut |report| {
                    last = report.nodes;
                    true
                },
            );
            last
        };
        // A table of one slot remembers almost nothing; a real one saves
        // work (about a quarter at depth 4, much more deeper down).
        let mut tiny = Table::new(0);
        let mut table = Table::new(1);
        let without = nodes(&mut tiny);
        let first = nodes(&mut table);
        assert!(first * 10 < without * 9, "{first} vs {without}");
        // Searching the same position again with what's already in the
        // table is quicker still.
        assert!(nodes(&mut table) < first);
    }

    /// Find the legal move written `text` (like "e2e4") in `position`.
    fn find_move(position: &Position, text: &str) -> Move {
        legal_moves(position)
            .into_iter()
            .find(|m| m.to_string() == text)
            .unwrap()
    }

    #[test]
    fn killers_come_after_captures_and_before_other_quiet_moves() {
        // White can take the pawn on d5 with the e4 pawn or the knight
        // on c3; every other move is quiet.
        let board = position("4k3/8/8/3p4/4P3/2N5/8/4K3 w - - 0 1");
        let killers = [
            Some(find_move(&board, "c3b5")),
            Some(find_move(&board, "e1f2")),
        ];
        let order: Vec<String> = ordered(&board, legal_moves(&board), None, killers, None)
            .iter()
            .map(|m| m.to_string())
            .collect();
        assert_eq!(&order[..4], ["e4d5", "c3d5", "c3b5", "e1f2"]);
        // Without killers those two quiet moves are nowhere near the front.
        let plain: Vec<String> = ordered(&board, legal_moves(&board), None, [None; 2], None)
            .iter()
            .map(|m| m.to_string())
            .collect();
        assert_ne!(&plain[..4], &order[..4]);
    }

    #[test]
    fn a_killer_that_captures_here_is_ranked_as_a_capture() {
        // Three captures: exd5 (pawn takes pawn), Nxd5 and Rxa5 (rook
        // takes pawn, the most expensive attacker, so last). Nxd5 may
        // have been a quiet killer elsewhere, but here it captures: it
        // keeps its place among the captures rather than dropping to the
        // killers' place after them.
        let board = position("4k3/8/8/p2p4/4P3/2N5/8/R3K3 w - - 0 1");
        let killers = [Some(find_move(&board, "c3d5")), None];
        let order: Vec<String> = ordered(&board, legal_moves(&board), None, killers, None)
            .iter()
            .map(|m| m.to_string())
            .collect();
        assert_eq!(&order[..3], ["e4d5", "c3d5", "a1a5"]);
    }

    #[test]
    fn only_quiet_refutations_become_killers_newest_first() {
        let board = position("4k3/8/8/3p4/4P3/2N5/8/4K3 w - - 0 1");
        let mut never = || false;
        let mut table = Table::new(1);
        let mut history = History::new();
        let mut searcher = Searcher {
            path: Vec::new(),
            nodes: 0,
            may_stop: false,
            stopped: false,
            should_stop: &mut never,
            table: &mut table,
            killers: [[None; 2]; MAX_PLY + 1],
            history: &mut history,
            quiets_tried: vec![Vec::new(); MAX_PLY + 1],
            pvs: true,
        };
        let (quiet, other_quiet, capture) = (
            find_move(&board, "c3b5"),
            find_move(&board, "e1f2"),
            find_move(&board, "e4d5"),
        );
        searcher.remember_killer(&board, 3, capture);
        assert_eq!(searcher.killers[3], [None, None]);
        searcher.remember_killer(&board, 3, quiet);
        searcher.remember_killer(&board, 3, quiet); // no duplicate
        assert_eq!(searcher.killers[3], [Some(quiet), None]);
        searcher.remember_killer(&board, 3, other_quiet);
        assert_eq!(searcher.killers[3], [Some(other_quiet), Some(quiet)]);
        // Other plies keep their own.
        assert_eq!(searcher.killers[2], [None, None]);
    }

    #[test]
    fn history_scores_rise_fall_stay_in_bounds_and_fade() {
        let board = position("4k3/8/8/3p4/4P3/2N5/8/4K3 w - - 0 1");
        let mv = find_move(&board, "c3b5");
        let mut history = History::new();
        history.adjust(Color::White, mv, 100);
        assert_eq!(history.score(Color::White, mv), 100);
        // Kept per side: Black's score for the same squares is untouched.
        assert_eq!(history.score(Color::Black, mv), 0);
        // However many refutations, the score never passes the limit, and
        // near the limit it grows more slowly.
        for _ in 0..10_000 {
            history.adjust(Color::White, mv, 5_000);
        }
        let near_top = history.score(Color::White, mv);
        assert!(
            near_top <= HISTORY_MAX && near_top > HISTORY_MAX - 100,
            "{near_top}"
        );
        for _ in 0..10_000 {
            history.adjust(Color::White, mv, -5_000);
        }
        assert!(history.score(Color::White, mv) >= -HISTORY_MAX);
        // Between moves the scores are halved; a new game clears them.
        history.clear();
        history.adjust(Color::White, mv, 300);
        history.age();
        assert_eq!(history.score(Color::White, mv), 150);
        history.clear();
        assert_eq!(history.score(Color::White, mv), 0);
    }

    #[test]
    fn quiet_moves_after_the_killers_follow_their_history_scores() {
        let board = position("4k3/8/8/3p4/4P3/2N5/8/4K3 w - - 0 1");
        let mut history = History::new();
        history.adjust(Color::White, find_move(&board, "e1d2"), 500);
        history.adjust(Color::White, find_move(&board, "c3a4"), 200);
        history.adjust(Color::White, find_move(&board, "e1f1"), -300);
        let killers = [Some(find_move(&board, "c3b5")), None];
        let order: Vec<String> =
            ordered(&board, legal_moves(&board), None, killers, Some(&history))
                .iter()
                .map(|m| m.to_string())
                .collect();
        // Captures, then the killer, then quiet moves by score; a move that
        // has failed before goes to the back.
        assert_eq!(&order[..5], ["e4d5", "c3d5", "c3b5", "e1d2", "c3a4"]);
        assert_eq!(order.last().unwrap(), "e1f1");
    }

    #[test]
    fn a_quiet_refutation_gains_history_and_the_quiet_moves_before_it_lose() {
        let board = position("4k3/8/8/3p4/4P3/2N5/8/4K3 w - - 0 1");
        let mut never = || false;
        let mut table = Table::new(1);
        let mut history = History::new();
        let mut searcher = Searcher {
            path: Vec::new(),
            nodes: 0,
            may_stop: false,
            stopped: false,
            should_stop: &mut never,
            table: &mut table,
            killers: [[None; 2]; MAX_PLY + 1],
            history: &mut history,
            quiets_tried: vec![Vec::new(); MAX_PLY + 1],
            pvs: true,
        };
        let (good, failed, capture) = (
            find_move(&board, "c3b5"),
            find_move(&board, "e1f2"),
            find_move(&board, "e4d5"),
        );
        searcher.update_history(&board, 3, good, &[failed]);
        assert_eq!(searcher.history.score(Color::White, good), 9);
        assert_eq!(searcher.history.score(Color::White, failed), -9);
        // A capture that refutes changes nothing: captures are ordered by
        // what they take, not by history.
        searcher.update_history(&board, 3, capture, &[failed]);
        assert_eq!(searcher.history.score(Color::White, capture), 0);
        assert_eq!(searcher.history.score(Color::White, failed), -9);
    }

    #[test]
    fn pvs_saves_work_without_changing_the_answer() {
        // With no table to carry results between move orders, plain
        // alpha-beta and PVS must agree on every score; PVS should get
        // there with fewer positions.
        let positions = [
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
            "r1bq1rk1/pp2bppp/2n1pn2/3p4/2PP4/2N1PN2/PP2BPPP/R2QKB1R w KQ - 0 8",
            "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1",
            "7k/8/8/8/8/8/R7/1R4K1 w - - 0 1",
        ];
        let (mut plain_total, mut pvs_total) = (0, 0);
        for fen in positions {
            let run = |pvs: bool| {
                let mut reports = Vec::new();
                search_with(
                    &[position(fen)],
                    5,
                    &mut Table::new(0),
                    &mut History::new(),
                    &mut || false,
                    &mut |report| {
                        reports.push(report.clone());
                        true
                    },
                    Features {
                        pvs,
                        aspiration: false,
                    },
                );
                reports
            };
            let (plain, with_pvs) = (run(false), run(true));
            for (a, b) in plain.iter().zip(&with_pvs) {
                assert_eq!(a.score, b.score, "{fen} depth {}", a.depth);
            }
            plain_total += plain.last().unwrap().nodes;
            pvs_total += with_pvs.last().unwrap().nodes;
        }
        assert!(pvs_total < plain_total, "{pvs_total} vs {plain_total}");
    }

    #[test]
    fn aspiration_windows_do_not_change_the_answer() {
        // With no table, a narrow window that holds gives the exact score,
        // and one that misses is widened until it does, so every depth's
        // score must match a full-window search. (Whether it saves work
        // depends on the table, which makes re-searches cheap: that is
        // measured with bench and SPRT, not here.)
        let positions = [
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
            "r1bq1rk1/pp2bppp/2n1pn2/3p4/2PP4/2N1PN2/PP2BPPP/R2QKB1R w KQ - 0 8",
            "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1",
            "r1b1kb1r/pppp1ppp/5q2/4n3/3KP3/2N3PN/PPP4P/R1BQ1B1R b kq - 0 1",
        ];
        for fen in positions {
            let run = |aspiration: bool| {
                let mut reports = Vec::new();
                search_with(
                    &[position(fen)],
                    6,
                    &mut Table::new(0),
                    &mut History::new(),
                    &mut || false,
                    &mut |report| {
                        reports.push(report.clone());
                        true
                    },
                    Features {
                        pvs: true,
                        aspiration,
                    },
                );
                reports
            };
            let (full, narrow) = (run(false), run(true));
            for (a, b) in full.iter().zip(&narrow) {
                assert_eq!(a.score, b.score, "{fen} depth {}", a.depth);
            }
            assert_eq!(full.len(), narrow.len(), "{fen}");
        }
    }

    #[test]
    fn no_move_when_checkmated() {
        // Fool's mate: White is checkmated.
        let mated = position("rnb1kbnr/pppp1ppp/8/4p3/6Pq/5P2/PPPPP2P/RNBQKBNR w KQkq - 1 3");
        assert_eq!(
            search(
                &[mated],
                3,
                &mut Table::new(1),
                &mut History::new(),
                &mut || false,
                &mut |_| { true }
            ),
            None
        );
    }

    #[test]
    fn mate_in_reads_scores() {
        assert_eq!(mate_in(MATE - 1), Some(1));
        assert_eq!(mate_in(MATE - 3), Some(2));
        assert_eq!(mate_in(-(MATE - 2)), Some(-1));
        assert_eq!(mate_in(150), None);
    }
}
