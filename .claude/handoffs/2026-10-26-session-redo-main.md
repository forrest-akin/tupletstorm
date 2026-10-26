# Handoff: finish the redo of `main` (next orchestrating session)

The redo of Phase 0 is built, reviewed, refined and verified clean. What's left is a few guard tests,
the cutover checks, and moving `main`. This file is for the next main session, so it can finish
without rereading the first one.

## State

- Worktree `R/.claude/worktrees/redo-main`, branch `redo-main` (unpublished): 13 commits, root
  `e7ab73b`, tip `1b9ea7d`. All gates pass at the tip; cargo test about 3.6 s, npm test about 1.1 s.
  `main` and `origin/main` are still at `21ba10c`; `phase0-original` (local) also points there and
  must survive the cutover.
- Verification at the tip (`.claude/work/redo-main/verify-core-r3.md`): 5M fuzzed links with no
  panic or unstable round trip; 160k links diffed against `docs/reference/rhythmTree.js` with no
  disagreement beyond the plan's and handoffs' accepted deviations.
- Seeding artifacts in R: `.claude/handoffs/2026-10-26-{redo,review,refine,session}-redo-main.md`
  and `.claude/plans/2026-10-26-redo-main.md`. The redo handoff and the plan are already in the
  root commit. Process logs: `.claude/work/redo-main/` (findings, impl/, review/, refine/, verify*,
  comparison.md, retro/).
- User decisions are in the redo handoff ("User decisions, 2026-10-26") and the refine handoff.
  These were added after those were written:
  - Before the cutover, add these guard tests (retrospective 3.2):
    - a round-trip property over custom lanes whose text includes `&=%#?.` and whitespace, folded
      into `feat: canonicalise the opened link`;
    - the decode property run on a 1 MiB thread, folded into `feat: open a v2 link and draw its
      bars`;
    - a 512/513-leaf boundary case, folded into the commit that adds `MAX_LEAVES_PER_BAR`.
    Everything stays inside the test budget.
  - Compile at folds (approved for CLAUDE.md "Commits", not yet written there): after a rebase or
    fold, run `cargo check` and `tsc` at each rewritten commit; the full tests still run only at
    the tip.
  - Shorter commit bodies (approved, applies to new commits): a body names the gates run and any
    deviations from the plan, not every test name.
- The claude-config changes the user approved are seeded separately, in
  `~/github.com/forrest-akin/claude-config/.claude/handoffs/2026-10-26-redo-retrospective.md`.

## Next

1. Guard tests, as above: one refine agent with a single `rebase -i` and edit stops at every
   target, plus a compile check at each rewritten commit. Then rerun the gates at the tip
   (plan, "Working rules" → "Gates").
2. Fold the review, refine and session handoffs into the root commit, which already holds the
   redo handoff and plan.
3. The plan's "Cutover" step 1: the clean-worktree tip check.
4. Manual click check by driving Chrome (user's choice): load the claude-in-chrome skill, start
   the dev server from the tip, open a fixture link, click a cell, see the address bar change,
   and read the console.
5. Show the user `.claude/work/redo-main/comparison.md` (offer it as a page) if they want it
   before `main` moves.
6. The plan's "Cutover" step 2: move `main` locally and keep `phase0-original`, then stop. The
   force push `git -C R push --force-with-lease=main:21ba10c origin main` is the user's to run, or
   to approve in that session.
