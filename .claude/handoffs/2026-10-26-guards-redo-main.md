# Handoff: guard tests, then the cutover of `redo-main`

Seeded by `.claude/handoffs/2026-10-26-session-redo-main.md`. `R` and `W` and the gates are as in
the plan's "Working rules" (`.claude/plans/2026-10-26-redo-main.md`). The generator gaps behind
the guards are in `R/.claude/work/redo-main/retro/retrospective.md` §3.2.

## Refine: guard tests and the handoffs (one agent, one rebase)

One `git -C W rebase -i --root` with edit stops at these commits, matched by subject:

| Commit | Change |
| --- | --- |
| `docs: add design references, the format reference and its fixture` | Add `.claude/handoffs/2026-10-26-{review,refine,session,guards}-redo-main.md`, each a byte-for-byte copy of the file in `R`. |
| `feat: open a v2 link and draw its bars` | The decode properties (`any_text_decodes`, `grammar_shaped_text_decodes`) run on a thread with a 1 MiB stack, the wasm default. A bar of exactly `MAX_LEAVES_PER_BAR` leaves decodes; one leaf more is rejected with the issue the core gives. |
| `feat: canonicalise the opened link` | A round-trip property over a custom lane whose text mixes `&`, `=`, `%`, `#`, `?`, `.`, whitespace and lane letters: the canonical round trip holds and the lane text comes back unchanged, except that a lone `.` reads as a rest. |

- A guard passes at its commit by construction. Its red is a mutation: break the behaviour it
  guards (each escaped character, the leaf limit), see the test fail for that reason, restore.
  Log each mutation and its result in the progress file.
- Resolve conflicts in later commits so each keeps its own intent.
- After the rebase, a compile check at each rewritten commit: `cargo check --workspace
  --all-targets --all-features`, and where `web/` exists, the web typecheck against that commit's
  own wasm package. Full tests run only at the tip.
- Test budget at the tip: Rust over 5 s lowers `CASES`; either suite over 10 s is a stop.
- Amend a body only where the change makes it wrong (times, counts) and add a line naming the
  guard. Bodies name gates and deviations, not every test.

## Verify: the tip (one agent)

1. The plan's "Cutover" step 1 in `R/.claude/worktrees/redo-tip`.
2. `git range-diff` from the pre-rebase tip `1b9ea7d` to the new tip shows only the changes above.
3. Each guard catches its mutation at the tip, and reverting the C1 and C2 fixes fails a test.
4. Manual click check in Chrome from the clean tip: open a fixture link, click a cell, see the
   address bar change, read the console.
5. Remove `redo-tip`.

## Cutover: move `main` (one agent, after the main session confirms)

The plan's "Cutover" step 3, with the `cmp` covering the plan and all five
`.claude/handoffs/2026-10-26-*redo-main.md` files. Then stop; the force push is the user's.
