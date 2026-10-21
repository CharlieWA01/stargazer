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

## 2026-10-20: CI builds the browser version on every PR (PR #4)
**Decision:** `rust-toolchain.toml` includes the WebAssembly target, and CI
builds the library for it on every PR.
**Alternatives:** check the browser build only when the portfolio tile is
built (Milestone 2 or later).
**Why:** the "runs in a browser" rule is easy to break by accident (a clock
read, a thread). A failing CI step catches it the day it happens instead of
months later. The first board code already compiles for the browser.
**Cost:** about half a minute more CI per PR, and a one-time download of the
WebAssembly standard library (rustup does it automatically).

## 2026-10-20: A move is from, to and promotion; tables built at compile time (PR #5)
**Decision:** a `Move` holds only its from-square, to-square and promotion
piece, exactly what UCI writes (`e7e8q`). Castling, en passant and double
pushes are recognised from the move itself when it's played. Knight, king
and pawn attack tables are built while Stargazer compiles.
**Alternatives:** extra flags on each move ("this is castling", "this is en
passant"), packed into 16 bits as fast engines do; building the tables when
the engine starts.
**Why:** the plain form reads like chess and converts to and from UCI with
no translation. Flags can be added later if `bench` shows a gain. Compile-time
tables cost nothing at startup, need no setup code, and work the same in
the browser build.

## 2026-10-20: CI perft goes to depth 4, not 3 (PR #6)
**Decision:** the per-PR perft test checks all six standard positions to
depth 4.
**Alternatives:** depth 3 (well under a second); depth 5 (minutes).
**Why:** a test bug proved depth 3 isn't enough. With the en passant
capture deliberately broken (the captured pawn left on the board), every
position passed at depth 3; only Kiwipete at depth 4 caught it
(4,085,601 instead of 4,085,603). Depth 4 catches it in about 10 seconds.
**Cost:** about 10 seconds more CI per PR.

## 2026-10-20: Search: alpha-beta core, only blunder-stoppers in Milestone 2 (PR #7)
**Decision:** negamax with alpha-beta and iterative deepening; in Milestone 2
only quiescence search, captures-first ordering, and mate and draw
detection. Every other search technique waits for Milestone 4 and SPRT.
**Alternatives:** add the well-known big gains (transposition table, null
move, late move reductions) straight away.
**Why:** the core is the standard, explainable starting point. Quiescence
search is what stops an engine hanging pieces, so it's needed to play at
all. Adding the rest before SPRT exists would mean guessing at their value
instead of measuring it, and measuring them makes good first tests of the
SPRT rig.

## 2026-10-20: Evaluation numbers are our own, tuned later (PR #7)
**Decision:** material plus tapered (middlegame/endgame) piece-square
tables, with values written by hand from chess principles and later fitted
to game results by our own Texel tuner.
**Alternatives:** the public "Simplified Evaluation Function" tables
(starting numbers borrowed); PeSTO's machine-tuned tables (strongest start,
but someone else's tuned work).
**Why:** Charlie wants it built and understood, and wants it tunable. Every
number can be explained, and the tuner is a small, real machine-learning
step whose gain SPRT can measure.
**Cost:** a weaker start than PeSTO until tuning.

## 2026-10-20: Simple time budget; search thread only for `stop` (PR #7)
**Decision:** spend about one twentieth of the remaining time plus most of
the increment per move. The native UCI layer searches on a second thread so
it can hear `stop`; the core stays single-threaded and clock-free.
**Alternatives:** time use that adapts to how complicated the position is
(Charlie's idea, parked for later); polling input from inside the search
(not possible in a browser).
**Why:** simple and safe now; smarter time use is a strength change, so it
waits for SPRT.

## 2026-10-20: Draw rules and how the search is driven (PR #8)
**Decision:** (1) insufficient material (king vs king, or a lone knight or
bishop) is scored as a draw, alongside repetition and the 50-move rule.
(2) Inside the search a position seen once before is already a draw.
(3) The search takes two callbacks, `should_stop` (asked every 2,048
positions) and `on_depth` (after each finished depth, which can end the
search), and never reads a clock itself. (4) Until clocks are handled, `go`
without a depth searches 5 half-moves deep.
**Alternatives:** (1) leave insufficient material out of Milestone 2;
(2) wait for a true threefold repetition; (3) pass a deadline into the
search; (4) refuse `go` without a depth.
**Why:** (1) it's cheap, and otherwise the engine thinks it's winning a dead
draw. (2) is the standard shortcut: a position that can be repeated once can
be repeated again, and waiting for three wastes search on lines that are
already drawn. (3) keeps the core free of clocks, which the browser build
handles differently from the native one. (4) lets a GUI get a move from
the engine now; depth 5 takes a fraction of a second in a release build.
**Cost:** none of note; (4) is temporary and goes away in step 3.

## 2026-10-20: The time budget in detail (PR #9)
**Decision:** a target of remaining/20 (or remaining/(movestogo+1), at most
20) plus 3/4 of the increment; a ceiling of 3 times the target; both capped
at the clock minus a 50 ms safety margin. Stop at the ceiling mid-depth;
start a new depth only before half the target. `go infinite` waits for
`stop` even when it runs out of depths. A bare `go` still searches 5 deep.
**Alternatives:** stop only at a fixed time (wastes the unfinished depth, or
overruns); predict each depth's length from the branching factor (more
code, same idea); no safety margin.
**Why:** the standard simple scheme. The half-target rule avoids starting a
depth that can't finish, and the ceiling covers a depth that runs long.
The margin covers the GUI or Lichess passing the move on. A bare `go` with
no limits is only typed by hand, where a quick answer is more useful than
thinking forever.
**Cost:** none. The numbers are first guesses; SPRT can tune them later.

## 2026-10-20: The first playable version is "Polaris" (PR #10)
**Decision:** the Milestone 2 engine (it plays full games under a clock) is
named Polaris, the first star in the release-name list.
**Alternatives:** wait for the first Lichess-rated version to use a name.
**Why:** Polaris is the guide star, a fitting name for the first version
that plays. Charlie chose it.
**Cost:** none. How releases are tagged and built is still open (DESIGN.md
section 6), so no tag or version bump yet.

## 2026-10-21: Strength tests run on AWS Fargate Spot (PR #11)
**Decision:** SPRT matches run as one-off containers on ECS Fargate Spot
(ARM, us-east-2), started from GitHub Actions, with the infrastructure in
cloud-homelab's Terraform.
**Alternatives:** a cheap always-on VPS ($5 to $10 a month, 2 cores, slow
for long tests); EC2 Spot (cheapest per hour, but a machine image to look
after and a sign-off per launch); Charlie's Chromebook (ties it up for hours).
**Why:** pay per second only while a test runs (about $0.20 an hour for 16
vCPU at Spot prices, so cents for an early test), nothing to patch, and it
fits cloud-homelab's "serverless and managed first" rule. Charlie also
wanted it on AWS for the resume.
**Cost:** a few cents a month idle (image and results storage), plus about
$0.10 to $1.20 per test. Spot can interrupt a test; it restarts.

## 2026-10-21: How a strength test is run (PR #11)
**Decision:** fastchess; a balanced opening book (UHO later); 8+0.08 games
(40+0.4 to double-check big changes); SPRT at 95% confidence with bounds
0 to 10 Elo now, 0 to 5 later, -5 to 0 for clean-ups; a manual Run button in
GitHub Actions; GitHub Actions builds both engines and uploads them to S3;
the verdict on the PR, games and logs in S3 for 90 days.
**Alternatives:** cutechess-cli; UHO from the start; longer games for every
test; starting tests from a PR label; building engines inside the container.
**Why:** each is the simpler or cheaper choice at our current strength (see
the Milestone 3 design doc in the journal). The container never needs
access to the private repo, and nothing costs money unless Charlie starts it.
**Cost:** none beyond the tests themselves.

## 2026-10-21: Testing and the Lichess bot run on different hosts (PR #11)
**Decision:** a hybrid. Fargate Spot is for testing (and later the Texel
tuner); the Lichess bot gets a steady host, chosen in Milestone 5.
**Alternatives:** one host for both.
**Why:** testing is bursty and can be interrupted; the bot holds live games
for hours and must not be. Regular Fargate always-on is about $29 a month
for 1 vCPU, over the budget; on a schedule it's about $4. The homelab mini
PC would be free.
**Cost:** none now.
