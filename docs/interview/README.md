# Interview stories

Short STAR stories (Situation, Task, Action, Result) drawn from building
Stargazer. Claude drafts the facts; rehearse them in your own words.
See `docs/DOCS-STANDARD.md`.

**Tags:**
- **Design**: architecture and trade-offs
- **Cost**: keeping running costs near zero
- **Testing**: proving correctness (perft) and strength (SPRT)
- **Process**: git, documentation, working with an AI agent
- **Security**: keeping secrets and account details out of public view

**How it was built, said plainly:** Stargazer is built with Claude Code as a
pair programmer. I set the goals and made the design decisions, own the
repo, CI and hosting, and review every pull request; Claude writes the Rust
and drafts the docs. Be ready to explain any part of the design and why it's
that way.

---

## Scoping before building: the browser demo that wasn't next
**Tags:** Design, Process, Cost

- **Situation:** My portfolio site had a "coming soon" tile promising you
  could play my chess engine in the browser. The obvious next task looked
  like "compile it to WebAssembly".
- **Task:** Decide what to actually work on next.
- **Action:** Checked the plan against the repo: the engine couldn't play a
  move yet, so there was nothing to put in a browser. Instead of building
  the demo, I turned "it must run in a browser" into a design constraint
  (no I/O or clock in the engine core, single-threaded in the browser) and
  started on the first real design question, how the engine stores a
  position.
- **Result:** The demo stays on track at about $0 a month (it runs on the
  visitor's computer), and the constraint costs almost nothing now instead
  of a rewrite later. It also made the engine easier to test.

## Proving the rules: a bug that hid at depth 3
**Tags:** Testing, Design

- **Situation:** A chess engine's move generator runs millions of times a
  second, and one rules bug (a wrong en passant, an illegal castle) makes
  it play nonsense. Unit tests can't cover every combination.
- **Task:** Prove the rules correct, cheaply enough to run on every PR.
- **Action:** Used perft: count every move sequence to a fixed depth from
  six standard positions and compare with published totals. Then checked
  the test itself by breaking the code on purpose. With en passant broken
  (the captured pawn left on the board), every position still passed at
  depth 3. Only one position, at depth 4, caught it (4,085,601 vs
  4,085,603). So CI runs depth 4 (about 10 seconds), and a deep run up to
  119 million positions happens before releases.
- **Result:** All six positions match exactly at full depth. The rules are
  proven, not assumed, and the test is known to catch the bug it's for.
  Lesson: test the tests.

## Two threads, one engine: thinking while listening
**Tags:** Concurrency, Testing, Design

- **Situation:** A chess engine has to think for a set time and still
  answer the GUI while it thinks (`stop`, "are you ready?"). The same
  engine also has to run in a web browser, where there's no clock to read
  inside the search and threads work differently.
- **Task:** Play under a real clock without ever losing on time, and keep
  the search core free of clocks and threads.
- **Action:** Kept the search single-threaded. It only asks a "should I
  stop?" question every 2,048 positions and reports after each depth. The
  native UCI layer runs it on a second thread and answers that question
  from a stop flag and a time budget: a target plus a hard ceiling, with a
  safety margin. When I checked the new tests by breaking the code, two of
  them couldn't fail: one depended on machine speed, one sent its input
  too fast to notice an early answer. I rewrote them, pulling the timing
  decisions into small pure functions and feeding the engine paced input,
  like a person typing.
- **Result:** Self-play under 10-second, 3-second and 1-second clocks with
  no losses on time and no illegal moves, plus tests that are proven to
  catch the bugs they're for. The browser build will reuse the same
  search unchanged. Lesson: a timing test that can't fail is worse than
  none, because it looks like protection.

## The engine that beat itself: proving a test rig is fair
**Tags:** Testing, Cost, Process

- **Situation:** I built a cloud rig that plays two versions of the
  engine against each other on AWS Fargate Spot and decides with a
  statistical test (SPRT) whether a change made it stronger. Its first
  job was a sanity check: the engine against an identical copy, where the
  only right answer is "no difference". It came back +25 Elo twice, and
  the second time the test declared the copy stronger.
- **Task:** Find out whether the rig was biased before trusting it with a
  single real change, without spending much.
- **Action:** Treated it as an experiment, not a hunch. Added two
  things to the rig: a fixed number of games, so a test couldn't stop at
  a lucky moment, and per-move records of each engine's speed, depth and
  response time. Then ran 2,000 games of the engine against itself, about
  25 cents of Spot time. It came out +7.3 ± 7.2, borderline. My
  hypothesis was that one engine got more CPU time, so I compared about
  92,000 moves per side: speed, depth and time use were identical, which
  ruled it out. A second 2,000-game run gave +1.0 ± 7.5.
- **Result:** The rig is fair (about +4 ± 5 over 4,000 games, which
  includes 0), and the false pass was chance plus stopping early, which
  the test's 5% error rate allows. Along the way the per-move data
  showed the engine wasting up to a second on some moves, now the first
  improvement to test. Lesson: check the measuring tool with a case where
  you know the answer, and change one thing at a time when it disagrees.
- **Follow-up (2026-10-23):** that wasted time was the first real change
  tested on the rig. Keeping the best move from a search that runs out of
  time, about a dozen lines of code, passed at +44 ± 22 Elo in 503 games.
  The data collected to check the rig found the engine's first big gain.
- **Follow-up (2026-10-24):** the second change, a transposition table
  (the engine remembering positions it has already searched), passed at
  +136 ± 37 Elo in 235 games. Before trusting it, a speed check on the
  first half showed a 10 to 12% slowdown; the cause was the position
  record growing past 256 bytes, fixed by packing four castling flags
  into one byte. Measure the cost of each half, not just the total.
- **Follow-up (2026-10-24):** the third, killer moves (trying first the
  quiet moves that just refuted a neighbouring position), passed at +45 ±
  23 Elo in 588 games. Three changes, about +225 Elo in total, each one
  measured on its own before it merged.
- **Follow-up (2026-10-24):** history moves saved only 1 to 4% of the
  work in the benchmark, which starts every position fresh, yet passed
  at +25 ± 16 Elo in real games, where what it learns carries from move
  to move. A benchmark measures what it can see; the match is the test.

## The idea that lost: when a good hypothesis fails the test
**Tags:** Testing, Process, Judgment

- **Situation:** After a run of gains (the engine went from about 1900
  to about 2180 against Stockfish in a day), I reworked how it spends
  its clock. The second rule was a textbook idea from strong engines:
  once the same best move has held for several depths of search, stop
  thinking and play it. On one sample position it cut a 12.5-second
  move to 4.1 seconds.
- **Task:** Keep it only if it makes the engine stronger, measured with
  the same statistical test (SPRT) as every other change.
- **Action:** Ran the test against the previous version. It failed
  clearly: -24.5 ± 19.4 Elo over 626 games, our first loss. Instead of
  tweaking numbers until it passed, I looked for why: the first few
  depths finish in microseconds and nearly always agree, so "settled for
  4 depths" was true on almost every move, and the engine routinely
  stopped after a quarter of its planned time. Dropped the rule, kept
  the write-up, parked a milder version for later, and tested the next
  rule on its own.
- **Result:** The engine didn't get weaker, because nothing merges
  without passing. The one-position measurement that looked like a win
  (a third of the time) was exactly the effect that lost games. Lesson:
  a hypothesis is only as good as its test, a failed one is still data,
  and "it looks faster" is not "it plays better".

## Going open source: picking a licence, and checking what goes public
**Tags:** Process, Security, Design

- **Situation:** Stargazer was a private repo. For the Lichess bot I
  wanted a deploy approval step, which on my GitHub plan needs a public
  repo, and I wanted the engine open source anyway, on the portfolio.
- **Task:** Make it public without leaking anything, under a licence that
  matches what I want: open, but nobody takes a fork private.
- **Action:** Compared three licences. MIT lets anyone reuse it, even in a
  closed product. GPL-3.0 means anyone who hands out a modified version
  must publish their source under the same terms. AGPL-3.0 also covers a
  modified copy run only on someone's server. I chose GPL-3.0, the norm
  for chess engines (Stockfish uses it), accepting the server gap as the
  trade for the conventional choice. Separately, I had every commit on
  every branch scanned (104) for account IDs, ARNs, keys, tokens and
  emails: the code was clean. The leak was somewhere easier to miss: the
  GitHub Actions logs, which go public with the repo. Three CI settings
  held the AWS account ID as plain variables, and GitHub prints variables
  in logs. They became secrets (which GitHub masks), each workflow now
  masks the account ID after signing in to AWS, and the old run logs that
  showed it are deleted before the switch.
- **Result:** The repo goes public under GPL-3.0 with nothing sensitive in
  the code, the history or the logs, at no cost. The lesson I'd repeat:
  "public repo" means the CI logs too, not just the code.


## Not taking the recommendation: pacing a game with no increment
**Tags:** Judgment, Process, Design

- **Situation:** In a 5-minute game with no increment, my engine was
  winning against Stockfish until move 44, then lost the endgame on
  about a second a move: it spent a fixed twentieth of its clock each
  move, so the clock shrank by 5% a move and was nearly gone when the
  hard part came.
- **Task:** Decide how the engine should spread its time. Claude (the
  AI writing the code with me) laid out three options and recommended
  the simplest: spend a fortieth instead of a twentieth.
- **Action:** I didn't take it. My reasoning as a player was that the
  middlegame needs the most time, then the endgame, then the opening.
  A fixed fraction doesn't know which phase it's in. I asked for the two
  "estimate the moves left" versions to be compared side by side on my
  actual game: one by move number, one by the material still on the
  board. I chose material: it thinks hardest when the pieces come off
  and the game changes character, and it isn't fooled by a game that
  trades down early.
- **Result:** Both versions roughly double the endgame thinking time
  compared with before, and the one I picked is tested on its own at
  10 seconds + 0, with the move-number version kept for a head-to-head.
  Lesson: a recommendation optimised for simplicity isn't always the one
  that matches the domain; bring your own knowledge, ask for the
  comparison, and then let the test decide.
