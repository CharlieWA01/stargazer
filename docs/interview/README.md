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
