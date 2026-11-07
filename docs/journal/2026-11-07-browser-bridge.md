# 2026-11-07: The browser bridge (Stargazer in a web page)

## Goal
Let visitors play Stargazer on the portfolio site, with the engine running
on their own computer. This session is the engine's half: a small bridge
that a web page can load as WebAssembly, and a `.wasm` file in every
release. The page itself is a portfolio PR.

## What was done
- Charlie picked from the plan (thread "Build playable browser Stargazer",
  2026-11-07): plain exported functions instead of a glue library, the
  `.wasm` committed into the portfolio repo, our own board with an
  open-licence piece set, **full strength only** (difficulty levels parked
  until they can be done properly), and a browser test on the page.
- New package `web/` (`stargazer-web`), a second member of a Cargo
  workspace next to the engine. The engine core didn't change at all:
  the 2026-10-20 rules (no clock, no input or output in the core) meant
  everything the page needs was already there.
  - `web/src/game.rs`: a game in progress. Legal moves, whose turn it is,
    and how the game ends: checkmate, stalemate, and the three draws a
    page has to call itself (the third repetition, fifty moves,
    insufficient material). Moves come back in ordinary chess notation
    (`Nbd2`, `exd6`, `O-O`, `e8=Q#`) for the move list. Thinking takes a
    number of milliseconds and the page's clock, and uses the same
    "don't start a depth past half the time" rule as `go movetime`.
  - `web/src/lib.rs`: each function answers with a short JSON text, and
    the `.wasm` exports `version`, `new_game`, `play`, `think` and `state`,
    plus `input` and `output`, which say where in WebAssembly's memory the
    page writes its text and reads the answer. The page's clock is the one
    imported function, `now`.
- `web/tests/smoke.mjs`: loads the built `.wasm` in Node the way the
  page's worker will, plays a move, lets the engine reply, and checks the
  reply is legal and on time. CI runs it on every PR.
- The `Release` workflow builds `stargazer.wasm` and attaches it (with its
  SHA-256) to every release. Version bumped to 0.2.0 so Charlie can tag
  the first release that has it.

## Problems and fixes
- Thinking needs the clock in two places (inside a depth and between
  depths), and Rust won't hand the same changeable function to both. The
  clock only reads the time, so it's passed as a read-only function
  (`Fn`) instead.
- Repetition: inside the search, one repeat already counts as a draw. A
  real game needs the third occurrence, so the bridge counts positions
  itself. The page draws at once on the third repetition and on move 50,
  as Lichess does, rather than asking someone to claim it.

## How it was verified
- `cargo test --workspace`: 22 new tests (checkmate, stalemate, each draw,
  mate on the fiftieth move still counting, notation for captures,
  castling both ways, en passant, promotion, and which of two pieces
  moved; a mate in one found inside its time; a lone legal move played
  without thinking; a quick game against itself to the end).
- Proved the tests can fail: made checkmate print `+` instead of `#`;
  three tests failed. Restored.
- `cargo clippy` for both the native and WebAssembly targets.
- The Node smoke test: `ok: Stargazer 0.2.0, replied Nc6 (depth 7,
  -0.05) in 180 ms`.
- Speed, one second per move: from the start position the `.wasm` in Node
  finished depth 9 (the native engine also finished depth 9); in Kiwipete
  depth 6 (native: 6). Same depths, so the browser version is about as
  strong per second on a desktop. Phones will be slower.
- The `.wasm` file is 141 KB.

## Impact
- Nothing changes for the Lichess bot (it stays on v0.1.0 until Charlie
  moves it).
- One more package to build, but no new crates: the engine is still
  dependency-free.
- Next: Charlie tags v0.2.0; the portfolio PR copies its `stargazer.wasm`.

## Afterwards (same day)
- Charlie merged #47 and tagged v0.2.0. The first tag landed on v0.1.0's
  commit: the laptop's copy of `origin/main` hadn't been fetched since
  then. The `Release` workflow's version check would have refused it, so
  nothing wrong was published; Charlie deleted it and tagged again after
  `git fetch origin`. Both release runbooks now say to fetch first.
- The released `stargazer.wasm` has the same SHA-256 as the one built in
  this session from the PR branch: the build is reproducible.
- The play page went live (portfolio #8), then gained drag and drop
  (portfolio #9). Charlie checked it with a mouse and a finger on a
  laptop, and on a phone.
- The Lichess bot still runs v0.1.0. v0.2.0 also carries the time
  management change made since then (sudden-death pacing: the clock is
  shared over the moves the game probably has left; `bench` is the same,
  343857), so moving the bot to it is a separate deploy
  (`docs/runbooks/lichess-bot.md`).
