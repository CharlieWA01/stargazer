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
**Amended 2026-10-23:** a depth stopped at the ceiling no longer throws
away its finished root moves (see "How Milestone 4 is ordered").

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

## 2026-10-21: How the test container is built (PR #11)
**Decision:** a Debian 13 ("trixie") image with fastchess compiled from a
pinned release and the opening book downloaded from a pinned commit and
checked by hash. The engines are normal (glibc) builds from GitHub's Ubuntu
24.04 machines. The run script is Python with no extra packages. Games that
are clearly decided are adjudicated with Stockfish's testing rules.
**Alternatives:** Debian 12 with fully static (musl) engines, which run on
any Linux; a prebuilt fastchess download; no adjudication.
**Why:** the static build searched about 30% slower (390k vs 560k
positions a second), which would quietly make every test shorter than
8+0.08 in practice; Debian 12's C library is too old for engines built on
Ubuntu 24.04. Building fastchess ourselves works the same on ARM and x86.
Adjudication saves time in long lost or dead-drawn endgames and doesn't
change who's stronger.
**Cost:** none. Building fastchess adds about a minute to an image build,
which happens only when `rig/` changes.

## 2026-10-21: The Run SPRT workflow waits 30 minutes, then hands over (PR #12)
**Decision:** "Run SPRT" waits up to 30 minutes for a verdict (a setting
on each run, 0 to 100). After that, a separate "SPRT report" workflow,
started by hand with the run id, posts the verdict.
**Alternatives:** wait for the whole test, up to about 5.5 hours (the
original plan); never wait, and always press Report.
**Why:** Stargazer is a private repo, and GitHub bills private repos for
Actions time even while a job only waits: the free plan has 2,000 minutes
a month, and waiting out whole tests would use them in about six tests.
Changes that make the engine worse usually fail within 30 minutes, so
those verdicts still arrive on their own.
**Cost:** at most about 40 Actions minutes per test, about 1 per report.
Recommended by Claude while Charlie's answer was pending; change it on the
form or here if Charlie picks otherwise.

## 2026-10-21: How the workflow builds for ARM (PR #12)
**Decision:** the engines are cross-compiled for ARM on GitHub's ordinary
x86 machines, and the container is built for ARM with QEMU (an emulator),
only when `rig/` has changed. Images are tagged with a fingerprint of
`rig/`; the tag `current`, which the AWS task runs, is moved to it before
each test.
**Alternatives:** GitHub's ARM machines (to our knowledge free only for
public repos); building the container on every run.
**Why:** cross-compiling Rust is quick and reliable; emulated builds are
slow (about ten minutes) but rare, since the container seldom changes.
**Correction (2026-10-21, after the first run):** the emulated build took
31 minutes, almost all of it compiling fastchess. fastchess is now
cross-compiled too; only installing the final image's packages runs
under emulation.
**Cost:** none beyond Actions minutes.

## 2026-10-22: The rig is accepted as fair (#15)
**Decision:** the test rig is trusted for strength tests. The +25 Elo
self-test results of 2026-10-21 are treated as bad luck, not a bug.
**Evidence:** two fixed 2,000-game self-matches on AWS, `main` against
`main`: +7.3 ± 7.2 and +1.0 ± 7.5 (about +4 ± 5 together). The game
files show identical speed, depth and time use for both copies.
**Alternatives:** keep investigating fastchess's handling of the first-
and second-listed engines (no mechanism found to look for); run every
future test twice with the engines' order swapped (doubles the cost).
**Why:** with identical conditions on every measured number and a
combined result that includes 0, more digging would chase noise. The
earlier false pass is explained by chance plus a test that stops the
moment it crosses the line, which SPRT's 5% error rate allows for.
**Practice from now on:** re-run a 2,000-game self-match whenever `rig/`
or the AWS setup changes, and treat a pass that is only just over the
line with suspicion (Charlie: "something to keep an eye on").
**Cost:** about 25 cents and 50 minutes per self-match.


## 2026-10-23: How Milestone 4 is ordered, and the first step
**Decision:** (1) Milestone 4 goes step by step, one PR and one SPRT test
each: unfinished-depth fix, transposition table, killer and history moves,
PVS and aspiration windows, null-move pruning, late move reductions and
check extensions, a time-management rework, evaluation terms, Texel
tuning. (2) First step: when time runs out partway through a depth, play
the best move among the root moves already finished (Charlie chose this,
option A). (3) A failed SPRT for this step means it's parked, not rerun.
**Alternatives:** (2) don't start a depth that can't finish (option B);
rework all of time management first, which Charlie asked about.
**Why:** (1) the search tricks bring the biggest gains, and each changes
how fast a search grows from one depth to the next, which time rules have
to be tuned against, so the time rework waits until they're in. (2) is
the standard fix, a few lines, and leaves the clock rules unchanged, so
the test measures only this. (3) the gain is probably small; rerunning a
failed test until it passes would turn SPRT's 5% error into much more.
**Cost:** none beyond the test (about $0.10 to $1.20).

## 2026-10-23: Stay on Fargate Spot; retry a refused start
**Decision:** strength tests stay on Fargate Spot. When AWS declines to
start a test, the workflow prints AWS's reason and retries twice, a minute
apart. The ARM cross-compiler's packages are kept in GitHub's cache.
**Alternatives:** regular (on-demand) Fargate for every test, or as an
automatic fallback when Spot has no room (needs a cloud-homelab change
and an on-demand vCPU quota).
**Why:** Charlie's call. The failure that prompted it wasn't a reclaim,
and Spot's problems so far (two reclaims, one refused start, in six runs)
each cost only a rerun. On-demand is roughly three times the price.
**Cost:** none. Revisit if Spot keeps failing.

## 2026-10-23: Transposition table design
**Decision:** (1) built in two PRs: Zobrist fingerprints first (no change
to the search; bench must stay equal), then the table. (2) Our own fixed
random numbers (SplitMix64, fixed seed, made at compile time). (3) 16 MB
by default with a UCI `Hash` option (1 to 1024 MB), the same size in the
browser. (4) Slots hold fingerprint, best move, score, depth and bound
type; mate scores stored relative to the position. (5) Always replace
when full. (6) Stored move tried first everywhere; stored score used to
end a search early only when deep enough, never at the root; table kept
between moves, cleared on `ucinewgame`. (7) Castling rights packed into
one byte so `Position` stays 256 bytes.
**Alternatives:** one PR; a random-number crate; a bigger default table;
depth-preferred or two-entry replacement; four `bool`s for castling.
**Why:** (1) proves the fingerprints right before anything relies on
them. (2) keeps the engine core dependency-free and bench repeatable.
(3) about a million slots, plenty at our speed, and the size lichess-bot
and GUIs expect to be able to set. (5) simplest, usually within a few Elo
at this stage; smarter replacement is a later SPRT test. (7) the
fingerprint pushed `Position` over 256 bytes and cost 10 to 12% speed;
packed rights bring it back to about 4%. Charlie approved (1) to (6).
(8) 2a merges without its own SPRT; 2b's test is run against the `main`
before 2a, so it measures both together (Charlie, after the 4% was found).
**Cost:** about 4% speed until the table uses the fingerprints; 16 MB of
memory per engine.

## 2026-10-24: Claude starts standard SPRT tests without asking
**Decision:** Claude may start a standard SPRT on its own: bounds 0 to 10
(a strength change) or −5 to 0 (a clean-up), at 8+0.08 on 16 vCPU of
Fargate Spot, one at a time. Anything else (other bounds, a longer time
control, more CPUs, regular Fargate) is still asked first.
**Alternatives:** asking before every test, as until now.
**Why:** each standard test costs cents and has the same shape every
time; waiting for a yes left finished PRs idle for hours. Charlie, in
the Milestone 4 thread.
**Cost:** a few cents of Spot time per test that Charlie didn't
individually approve; the $10 org budget alert still applies.

## 2026-10-24: Killer and history moves, in two PRs
**Decision:** (1) killers and history are built as two PRs, killers
first, each with its own SPRT (Charlie's choice). (2) Killers: two per
ply, quiet moves only, newest first, tried right after the captures,
kept across the depths of one search and cleared at each `go`; a killer
that captures in the new position is ranked as a capture. (3) History:
by side, from and to square, bonus depth × depth; the details (halving
between moves, lowering failed quiet moves) are settled with its PR.
**Alternatives:** (1) one PR and one SPRT for both; (2) one killer per
ply, or killers kept between moves; ordering killers before the weakest
captures (needs a way to tell a losing capture, which we don't have).
**Why:** (1) killers are the larger, safer gain; history often does
little until late move reductions use it, and a separate test shows
whether it earns its place now. (2) is the textbook form: two slots
catch a second refutation without letting a stale one linger. Captures
stay first because without an exchange evaluator every capture looks
promising.
**Cost:** one extra SPRT (cents). Killers cost about 1 KB of memory and
about 2% of speed per position, against 9 to 15% fewer positions.

## 2026-10-24: History details
**Decision:** (1) history scores are halved at the start of each search
and cleared on `ucinewgame`; (2) when a quiet move refutes a position,
the quiet moves tried before it there lose the same amount it gains
(both Charlie's choice); (3) scores are kept within ±16,384 by shrinking
each change as a score nears the limit; (4) the UCI layer keeps the
history and the transposition table together as the search's memory,
and bench gives each position a fresh history.
**Alternatives:** (1) clear between moves; (2) only reward the
refutation; (3) a hard cap, or halving everything when one score gets
too big; (4) a history inside the transposition table.
**Why:** (1) and (2) are what most engines settled on: old patterns fade
without being lost, and the failures are what sharpen the order. (3) is
the standard way to keep scores bounded without a sudden jump. (4) the
two are separate ideas (positions vs moves) that share a lifetime.
**Cost:** 32 KB of memory; about 1.5% of speed per position, for 1 to
4% fewer positions in bench (which can't show the carry-over between
moves, where history should help most).

## 2026-10-24: PVS and aspiration windows, in two PRs
**Decision:** (1) principal variation search and aspiration windows are
built as two PRs, PVS first, each with its own SPRT (Charlie's choice).
(2) PVS everywhere in the main search, root included, not in
quiescence. (3) A root move whose quick test says "better" but whose
proper search runs out of time is still played. (4) The search keeps a
switch to turn PVS off, used only by a test that checks the scores
don't change.
**Alternatives:** (1) one PR; (2) PVS only away from the root; (3) keep
the old best until the proper search finishes; (4) trust the existing
tests alone.
**Why:** (1) PVS is the bigger, safer gain; aspiration windows are
smaller and affect time use, so they're measured alone. (2) the root is
where ordering is best, so it gains most there. (3) the quick test
already proved the move beats the old best: the same reasoning as
keeping an unfinished depth's moves (#16). (4) PVS should change only
the work, never the answer, and comparing the two is the direct proof.
**Cost:** one extra SPRT; a few lines kept only for the test.

## 2026-10-24: Aspiration window size
**Decision:** (1) from depth 5, the root is first searched within ±25
centipawns of the last depth's score (Charlie's choice). (2) A miss
doubles the window on the side that missed; past 400 centipawns the
full window is used. (3) Mate scores always get the full window. (4) If
time runs out after a fail high, the move that failed high is played;
after a fail low, the last finished depth's move stands. (5) The
test-only PVS switch becomes a small set of switches, so a test can turn
aspiration windows off too.
**Alternatives:** (1) a wider or narrower start (±15, ±50); (2) widen
both sides on a miss, or jump straight to the full window; (3) narrow
windows around mates too; (4) always keep the last depth's move.
**Why:** (1) and (2) are the common starting point; the numbers can be
tuned by SPRT later. (3) mate scores jump by large amounts between
depths, so a narrow window would just miss. (4) a fail high already
proves the move beats the old best, the same reasoning as PVS at the
root (#23); a fail low proves nothing about any move.
**Cost:** a few lines; re-searches on a miss, which the transposition
table makes cheap. Bench saves 6% at depth 5 but under 2% at 6 and 7,
so the SPRT decides whether it stays (it passed: +7.8 ± 6.0 Elo over
6,282 games).

## 2026-10-24: Matches against Stockfish on the test rig
**Decision:** (1) a "Run match" button on the AWS test rig plays
Stargazer against Stockfish at a set strength (`UCI_Elo`) to estimate
its rating (Charlie's choice). (2) It runs the SPRT workflow in a match
mode instead of copying it: one workflow file holds the build, upload
and start steps for both. (3) The opponent is Debian's `stockfish`
package, installed in the same container. (4) Defaults: 400 games at
10+0.1 on 16 vCPU, one Stockfish level per run; Stockfish on one thread
with a 16 MB table, like Stargazer. (5) Claude may start a match on its
own when it uses at most 16 vCPU and takes under an hour; anything
bigger is asked first.
**Alternatives:** (1) a script on the Chromebook (free, but an hour or
more for a couple of hundred games), or both; (2) a separate copy of
the workflow; (3) Stockfish's own release binary, pinned by hash, or
built from source; (4) a "ladder" of several levels per run, or longer
games; (5) ask every time.
**Why:** (1) the rig already has everything (fastchess, the book,
Fargate, the report), so a real number takes minutes and pennies.
Watching games at home needs nothing built: PyChess and `apt install
stockfish` (runbook). (2) one place to fix the steps. (3) Debian builds
it for ARM, and the report records the exact version; Stockfish's own
ARM downloads are made for Android, and building it adds minutes to
every image build. (4) simple first; a ladder can come later. (5) same
size and cost as the standard SPRTs already approved.
**Cost:** a few cents per 400-game match; the rig image grows by
Stockfish (a few tens of MB with its network file) and is rebuilt once.
The rating is rough (±30 Elo at 400 games, and Stockfish's levels were
set at longer games).
