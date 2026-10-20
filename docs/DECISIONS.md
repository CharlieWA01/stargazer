# Decisions

Why Stargazer is built the way it is. Newest at the bottom. Each entry: what
was decided, what else was considered, the trade-offs, and why. A decision
that changes later gets marked superseded, not deleted, so nobody (human or
Claude) re-argues a settled question or undoes something without knowing why
it was there. Format: see `docs/DOCS-STANDARD.md`. The current state of the
architecture is in `docs/DESIGN.md`.

## 2026-10-18: Rust, UCI, from scratch (PR #1)
**Decision:** write the engine in Rust, speak UCI, write every part from
scratch, and call it Stargazer.
**Alternatives:** C++ (what most engines use), Python (Charlie reads it, but
far too slow for search); the older XBoard protocol; starting from an
existing engine.
**Why:** Rust is fast, and its strict compiler catches many bugs before the
code runs, which matters when the architect isn't reviewing code line by
line. UCI is what GUIs, testing tools and lichess-bot speak. Building from
scratch is the point of the project.
**Reconfirmed 2026-10-20:** Rust also compiles cleanly to WebAssembly (see
below).

## 2026-10-20: Goal: about 1800 on Lichess, with code Charlie understands (PR #2)
**Decision:** "done" is an engine with a Lichess rating around 1800 whose
design Charlie can explain. Understanding first, strength second. 2000 is a
possible later push.
**Alternatives:** chase maximum strength (NNUE, multithreading early); ship
a weak but playable engine to the website as fast as possible.
**Why:** a classic engine (alpha-beta search and a hand-written evaluation)
reaches 1800 with no neural network, and all of it is code that can be read
and explained in an interview. 2000 needs more search and evaluation work
and tuning, but no change of design.
**Cost:** none in money. Strength testing needs CPU time (local or homelab).

## 2026-10-20: NNUE is a stretch goal, and not "the ML project" (PR #2)
**Decision:** hand-crafted evaluation first. The evaluation is designed as a
swappable piece so a neural network (NNUE) could replace it later.
Charlie's machine-learning project will be a separate one.
**Alternatives:** make Stargazer's NNUE the ML project.
**Why:** NNUE is a tidy ML problem, but niche: it doesn't teach the data,
deployment or AWS ML skills employers ask for, it usually means using
someone else's trainer (against "from scratch"), and 1800 doesn't need it.

## 2026-10-20: It must also run in a browser (PR #2)
**Decision:** Stargazer compiles to WebAssembly as well as a native program.
The engine core never reads input, prints output or reads the clock itself;
the browser build is single-threaded; any JavaScript-bridge crate stays
outside the core.
**Alternatives:** native only, and run the engine on a server (such as a
Lambda per move) for website visitors.
**Why:** visitors can play it on the portfolio site with the engine running
on their own computer: no server, about $0, nothing to protect from abuse.
The rules also make the engine easier to test.
**Trade-offs:** the browser version is a little weaker than the
multithreaded native one.

## 2026-10-20: Board representation: hybrid (PR #2)
**Decision:** 12 bitboards (one 64-bit number per piece type and colour)
plus a 64-square array, kept in step. All shift-and-mask code lives in one
small, tested module with chess-named functions.
**Alternatives:** a mailbox (64-square array) only: easiest to read, but a
few times slower (rough estimate) and hard to switch away from later.
Bitboards only: as fast, but "what's on this square?" means checking twelve
numbers.
**Why:** bitboard speed and the industry-standard design, while search and
evaluation still read like chess. Perft catches the two copies disagreeing.
**Cost:** about 64 bytes more per position and a few more lines in "make
move" and "unmake move".

## 2026-10-20: Portfolio tile: play in the browser, and link to Lichess (PR #2)
**Decision:** the portfolio tile opens a page where visitors play Stargazer
in their browser, and links to the Lichess bot's profile for its real
rating. The page says the browser version is a bit weaker.
**Alternatives:** only one of the two; showing the live rating on the tile.
**Why:** the browser game is a one-click demo; a rating earned against
strangers is more convincing than anything on the site itself. A link
instead of a live rating keeps the site free of calls to other servers.

## 2026-10-20: Lichess bot hosting (PR #2)
**Decision:** run lichess-bot (an open-source Python bridge that speaks UCI
to the engine) on the planned homelab mini PC when Milestone 5 comes.
**Alternatives:** Charlie's own machine (only online while running); a
small always-on cloud server, about $5 to $15 a month (Lambda can't hold the
bot's all-day connection).
**Why:** no extra cost and no exposure: the bot only connects out to
Lichess. The account is registered and stays at zero games until then,
because only an account with no games can be upgraded to a BOT, and the
upgrade can't be undone.

## 2026-10-20: Adopt the shared documentation standard (PR #3)
**Decision:** Stargazer follows `docs/DOCS-STANDARD.md` like Charlie's other
repos: journal, this file, roadmap, portfolio page, interview stories,
history, the PR checklist and the `docs-check` and `docs-history` workflows.
`docs/DESIGN.md` stays as the architecture's current state.
**Alternatives:** keep decisions only in DESIGN.md's decision log.
**Why:** every repo reads the same way, and the record is ready for
interviews and for Claude's next session.

## 2026-10-20: Move generation: pseudo-legal, ray walking, copy-make (PR #4)
**Decision:** generate pseudo-legal moves and reject those that leave the
king attacked; find sliding-piece moves by walking rays; try moves by
copying the position.
**Alternatives:** fully legal generation (pins and checks worked out first);
magic bitboards or PEXT for sliding pieces; make/unmake.
**Why:** each choice is the simplest to write, read and explain, and fast
enough for the 1800 goal. Each faster alternative stays possible behind the
same functions, to be adopted only if `bench` measures a real gain, which
also gives a true before-and-after number. PEXT is ruled out by the browser
build.
**Trade-offs:** slower move generation than a tuned engine (rough guess:
ray walking is about half the speed of magics for sliding pieces).

## 2026-10-20: Perft in CI and before releases (PR #4)
**Decision:** the six standard perft positions at shallow depth in CI on
every PR; deep runs before each release, doubling as the `bench` command.
**Alternatives:** CI only (too slow at useful depths); a separate run only
(bugs reach `main` unnoticed).
**Why:** CI catches rule bugs on every change in under a minute; the deep
run catches the rare bug that only shows several moves in.

## 2026-10-20: CI builds the browser version on every PR (PR #5)
**Decision:** `rust-toolchain.toml` includes the WebAssembly target, and CI
builds the library for it on every PR.
**Alternatives:** check the browser build only when the portfolio tile is
built (Milestone 2 or later).
**Why:** the "runs in a browser" rule is easy to break by accident (a clock
read, a thread). A failing CI step catches it the day it happens instead of
months later. The first board code already compiles for the browser.
**Cost:** about half a minute more CI per PR, and a one-time download of the
WebAssembly standard library (rustup does it automatically).
