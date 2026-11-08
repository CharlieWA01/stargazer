# 2026-11-08: King safety

## Goal
Add the fourth and last evaluation term from the design (DESIGN.md
section 4, decided 2026-11-07): in the middlegame, a king behind its
pawns is safe and a king facing enemy pieces is not.

## What was done
`king_safety()` in `src/eval.rs` adds to the middlegame score only, so
it fades out as pieces come off and the king should come out and fight.
Three parts:

- **Pawn shield:** while the king is on its first two ranks, each of the
  three files around it earns +10 for a pawn of its own just in front of
  the king, or +5 for one a rank further on. A king castled on g1 behind
  f2, g2, h2 scores +30; push all three a rank and it's +15.
- **Open files:** a file among those three with no pawn of its own at
  all is a road for enemy rooks and queens: -15 each.
- **Attackers:** each enemy piece that reaches the king's square or a
  square next to it adds its weight (knight 2, bishop 2, rook 3, queen
  5). The danger is the total squared, halved, capped at 250: a queen
  alone costs 12, a queen and a knight 24, a queen, rook and knight 50.
  So one attacker barely matters, and an attack with several pieces
  matters a lot, which is how attacks on a king really work.

The attackers are counted inside `mobility()`: it already works out
where every knight, bishop, rook and queen reaches, so checking whether
that reach touches the enemy king's squares is one extra comparison per
piece. That's why king safety costs no measurable speed.

## Problems and fixes
- The first tests compared against the constants themselves (for
  example `3 * SHIELD_NEAR`), so with a constant set to 0 the expected
  value became 0 too and the test still passed. They now spell out the
  numbers (3 × 10 + 3 × 15), and each fails when its constant is 0.
- Two attacker test positions were wrong on the board: a rook on e8
  doesn't reach any square next to a king on g1 (e1 isn't next to it),
  and a queen on a2 does (along the second rank to f2). The rook went to
  d2 and the "too far" queen to a5.
- An older test, `the_king_hides_in_the_middlegame_and_centralises_in_the_endgame`,
  compared a king castled on g1 with **no pawns** against one on e3.
  With king safety, a castled king with three open files beside it and
  enemy rooks and queen about is no longer safe, which is right. The
  test now gives both kings the same three pawns (f2, g2, h2), which is
  what it meant: castled behind pawns beats walked out.

## How it was verified
- New tests: `a_king_behind_its_pawns_is_safer` (full shield, pushed
  shield, no shield, and Black's the other way),
  `a_king_that_has_walked_out_gets_no_shield`, and
  `attackers_near_the_king_add_up_faster_than_one_by_one` (a queen, then
  queen, knight and rook; a queen whose lines miss the king). Each was
  seen failing when its rule was broken (each number set to 0, the
  danger made linear, the shield applied on every rank, Black's shield
  looked for on the wrong side, the king's neighbours left out of the
  attack zone), then passing.
- 128 tests pass; `cargo clippy`, `cargo fmt`, the wasm builds and the
  browser smoke test are clean.
- Bench changes, as expected: 296359 nodes becomes **287042**. Speed,
  six runs each, alternating: median about 401,000 positions a second
  against 398,000 for main, the same within the noise.
- SPRT [0, 10] at 8+0.08 against main (`sprt-42432258847-1`):
  **passed**, +70.6 ± 28.9 Elo over 437 games (191 wins, 141 draws,
  105 losses; LLR 2.98), in about 13 minutes.

## Impact
About +71 Elo at no speed cost. All four evaluation terms from the
design are now done: mop-up was tried and dropped, and pawn structure
(+38), mobility (+74) and king safety (+71) together come to about
+180 Elo over the engine before them, measured one step at a time. Next: a Stockfish match to put a rating on the
whole step, then Texel tuning of every number at once.
