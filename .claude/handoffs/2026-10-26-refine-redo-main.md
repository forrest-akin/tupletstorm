# Handoff: refine the redo of `main`

The review of `redo-main` (`.claude/handoffs/2026-10-26-review-redo-main.md`) confirmed 16
blocking defects. Fix them, plus the adopted proposals below, by folding each fix into the commit
that introduced it. The branch is unpublished, so history is rewritten freely; nothing is pushed.

Finding details (evidence, file:line, suggested fix) are in `.claude/work/redo-main/review/`:
`core.md`, `client.md`, `design.md`, `history.md`, and the `*-verify.md` files. Each finding names its
introducing commit by its pre-refinement SHA; match commits by subject, since SHAs change as you
rebase.

## Fixes, by target commit

| Target commit (subject) | Fix |
| --- | --- |
| root `docs: add design references…` | `docs/milestones.md`: milestone 7's exit criteria gain "decode cost on hostile links measured and bounded" (user decision: accept the C7 cost until then). |
| `feat: open a v2 link and draw its bars` | C3 trim input as the reference does (with U+FEFF); D3 build the 16ths bar without `parse(..).expect`; H1 the body gives a reason for every npm package; D5/C10 `# Panics` docs on `Rat`; D11 bar tests assert through `groups()`, `to_string()` or `leaves()`; D13 `mod view` private, restating field docs dropped, `MAX_LEAVES_PER_BAR` `pub(crate)`; D14 `core.ts` binds the imports directly if it typechecks; W2 and W4 assertions for slot widths, the segment separator and the mirror's `segment`, `span` and default `grouping`. |
| `feat(core): explain rejected bars…` | C6 a line terminator in a bar is `NotABar`. |
| `feat: draw each lane's hits` | W1 the client indexes hits by code point, matching the core's chars; W2 cell width and separator assertions; W3 rest-cell text assertion and the issue level class; D7 `is_lane_key` and `KNOWN_KEYS` live in `score.rs` (no codec↔score import cycle). |
| `feat: show the groove's title and tempo` | W3 an untitled groove renders no heading (assertion). |
| `feat: canonicalise the opened link` | C2 custom-lane text percent-encodes `%`, `&`, `=`, `#` and `?`, and a lone `.` in a custom lane reads as a rest (the canonical round trip holds; the fixture is unaffected); C4 split `*N` only when the body after `:` stays non-empty; C12 trim swing as tempo is trimmed; H3 crate and module docs list what `Score` holds; D12/W5 drop the duplicate assertion; H7 the hand-edit sentence says to redo hand edits after regenerating. |
| `feat: cycle a cell's letter…` | D6 one function gives a lane's letters, used by `set_hit` and the view; H6 `docs/milestones.md` status names what milestone 2 has working and that Play waits on RFD 0004. |
| `feat(core): read nested splits…` | H4 split into a `refactor(core)` commit that moves `Cursor`, `int` and `MAX_SLOTS` to `node.rs`, then the feature; H2 and D9 segment, `Leaf::segment` and `Node::w` docs say segments count beat groups. |
| `feat: read and show beat groupings` | C5 a malformed grouping is `NotABar` before the meter's range checks; deferred here because `grouping` first exists in this commit: W4 `core.test.ts` asserts a default grouping is `null`, and C4's `1/4~3-3:*3` row (`grouping 3+3 doesn't add up to 1`) joins `reads_bar_repeats`. |
| `feat(core): keep tempo changes…` | D8 tempo changes get their own error type, so impossible `Issue`/`ParseError` pairings can't be built; H3 docs. |
| `refactor(core): read slot counts per lane` | D1 drop the dead `hits` parameter, and with it the `too_many_arguments` suppression. |
| `feat: draw lane overrides` | C1 reject an override path longer than the depth limit before walking it; C6 a line terminator in an override is `NotOverride`; D2 store each lane's grafted bar when an override is added, so `lane_leaves` needs no `expect` (this also removes the quadratic re-graft, C8); D4 the doc lands on the function it describes; H3 and H8 docs. |

Declined: W2's rule claim (refuted; its assertions are adopted above), C7 (accepted until
milestone 7), C8 beyond D2's fix, C11, D7's stricter override-key grammar (the reference accepts
the looser one), D10. H5: the RFD Sources lines stay (user decision).

## How

- Oldest target first. Stop at a target with
  `GIT_SEQUENCE_EDITOR="sed -i '' 's/^pick <sha> /edit <sha> /'" git -C <W> rebase -i <sha>^`, make its
  fixes, amend, then `git -C <W> rebase --continue`. Resolve conflicts in later commits so each keeps
  its own intent.
- A behaviour fix is test-first: a regression test that fails for the right reason at the target
  commit, then the fix.
- Update a commit's body wherever a fix makes it inaccurate (test lists, behaviour, dependency
  reasons). No attribution trailers.
- Run the target's own tests while fixing it, and the full gates and test budget at the tip after
  each rebase completes. Don't rerun the gates on every commit.
- Don't edit the plan or the earlier handoffs; record deviations in your progress file.
- No pushes. Leave `R/.zed/` and `R/web/.prettierrc` alone.
