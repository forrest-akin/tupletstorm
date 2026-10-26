# Handoff: review the redo of `main`

The redo branch `redo-main` (worktree `.claude/worktrees/redo-main`) holds 12 commits, `eaf8fac`
(root) to `7c4843e`, built from `.claude/plans/2026-10-26-redo-main.md` under
`.claude/handoffs/2026-10-26-redo-main.md`. Every gate is green at the tip; `cargo test` runs in
3.4 s and `npm test` in 1.1 s. Review it before `main` moves. Fixes come later, as fixups folded
into the commit that introduced each defect; this phase only finds and verifies.

## Inputs

- The plan and the redo handoff (its user decisions bind the code).
- `.claude/work/redo-main/findings-code.md`: what the Phase 0 port decided, including the panics
  and limit gaps found by probing it. The redo claims to fix them.
- `.claude/work/redo-main/impl/*.md`: each commit's progress log and its deviations from the plan.
- The commit bodies (`git -C <W> log`), and `docs/reference/rhythmTree.js` with the conformance
  fixture as the behavioural reference.
- `phase0-original` (local branch) is the old code, for comparison only.

## Known items to confirm or refute

- Custom-lane letters are written unescaped (as the reference does), so a custom lane holding `&`,
  `=`, `#` or `?` breaks `encode(decode(encode(s))) == encode(s)` (impl/5a.md).
- `Score::lane_leaves` has an `expect`, a second panic beyond the error model's one (impl/9b.md).
- `#[expect(clippy::too_many_arguments)]` on `Score::new` and
  `#[expect(clippy::suspicious_arithmetic_impl)]` on `Rat`'s `Div` (impl/8.md, impl/1.md).

## Severity

"Blocking" is a defect: wrong behaviour against the reference, the fixture or a stated invariant;
a commit that would not deploy alone; a doc that is untrue at its commit; or a broken rule in
CLAUDE.md or the handoff. Everything else is a "proposal", including any finding whose fix adds a
layer.

## Constraints

- No edits, commits or pushes in the repository or the worktree. Probes and scratch code go under
  the session scratchpad, never in the repository.
- Leave `R/.zed/` and `R/web/.prettierrc` alone.
