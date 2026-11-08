# 2026-11-07: Pawn structure

## Goal
Add the second evaluation term from the design (DESIGN.md section 4,
decided 2026-11-07): doubled, isolated and passed pawns.

## What was done
- `pawn_structure()` in `src/eval.rs` scores both sides' pawns and adds
  to the middlegame and endgame scores, so it fades with the material
  left like everything else:
  - **Doubled:** -10 middlegame / -20 endgame for each extra pawn on a
    file. Two pawns on one file can't protect each other, and the back
    one is stuck behind the front one.
  - **Isolated:** -10 / -15 for a pawn with no pawn of its own on the
    files beside it. No pawn can ever defend it, so pieces have to.
  - **Passed:** in the endgame, +10, 15, 25, 45, 75, 120 as the pawn
    stands on its 2nd to 7th rank, when no enemy pawn is in front of it
    on its own file or the files beside it. This sits on top of the
    square tables' bonus for advancing, so a passed pawn on the 7th
    rank is worth about 2.2 pawns in the endgame in all.
- The checks use whole files at once (bitboards): "any of my pawns on
  the files beside this one?" is one comparison, not a loop over
  squares.
- Mop-up, the first term, failed its test and was dropped (journal
  2026-11-07-evaluation-terms); pawn structure applies in nearly every
  game, so its effect should be much easier to measure.

## Problems and fixes
- The first doubled-pawn test still passed with the doubled penalty set
  to 0, because the pawns in it were also isolated, and the isolated
  penalty made the same difference. The isolated test compared a
  difference that is 0 when its penalty is 0. Both now use positions
  where only their own rule differs, and each was seen failing with its
  number set to 0, then passing.
- Two search tests were built on what the old evaluation happened to
  prefer, and broke when the evaluation changed:
  - one needed a position where depth 4 and depth 5 pick different
    moves; the old one now agrees at both, so it uses Kiwipete, where
    depth 4 likes Bxa6 and depth 5 dxe6;
  - one needed the transposition table to save more than 10% of the
    work at depth 4; it saves 8% there now, so it searches to depth 5,
    where it saves about 20%.
  Neither test's rule changed, only the position or depth it uses.

## How it was verified
- New tests: `doubled_pawns_score_lower_than_pawns_side_by_side`,
  `an_isolated_pawn_scores_lower_than_a_supported_one`,
  `passed_pawns_are_recognised_and_grow_as_they_advance` (including
  which squares count as "in front" for each colour), and a position
  with doubled, isolated and passed pawns on both sides added to the
  mirrored-positions test. Each was seen failing when its rule was
  broken (penalty set to 0, the passed-pawn path narrowed to one file,
  the rank counted from White's side for both colours), then passing.
- 122 tests pass; `cargo clippy`, `cargo fmt`, the wasm builds and the
  browser smoke test are clean.
- Bench changes, as expected for an evaluation change: 343857 nodes
  becomes **282819**. Speed, six runs each, alternating: median about
  419,000 positions a second against 435,000 for main, about 4% slower,
  under the 10% that would call for a pawn hash table (DECISIONS.md,
  2026-11-07).
- SPRT [0, 10] at 8+0.08 against main (`sprt-42414890759-1`):
  **passed**, +37.8 ± 20.4 Elo over 817 games (334 wins, 236 draws,
  247 losses; LLR 2.98), in about 20 minutes. The largest gain since
  null-move pruning, despite being 4% slower.

## Impact
The engine now sees weak pawns and dangerous passed pawns, not just how
far each pawn has advanced: about +38 Elo for about 4% of search speed.
The contrast with mop-up, tested the same day under the same bar, is
the lesson: a term that applies in almost every game shows up in a few
hundred games; one that almost never applies can't show up at all.
