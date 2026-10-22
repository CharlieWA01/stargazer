# Interview stories

Short STAR stories (Situation, Task, Action, Result) drawn from building
Stargazer. Claude drafts the facts; rehearse them in your own words.
See `docs/DOCS-STANDARD.md`.

**Tags:**
- **Design**: architecture and trade-offs
- **Cost**: keeping running costs near zero
- **Testing**: proving correctness (perft) and strength (SPRT)
- **Process**: git, documentation, working with an AI agent

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
