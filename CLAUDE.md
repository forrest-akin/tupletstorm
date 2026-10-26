# Tupletstorm — project guide for Claude

Tupletstorm is a drum and percussion notation system: write grooves with mixed
meters, beat-by-beat subdivisions, per-lane polyrhythms and nested tuplets, hear
them, and share them as links. It is planned as a small distributed system built
in phases (see `docs/milestones.md`). Phase 0 holds the design references, the
frozen format reference and its conformance fixture. Phase 1 builds the web
client and the core together, top-down from milestone 2.

## Repository map

| Path | What it is |
| --- | --- |
| `crates/rhythm-tree` | The shared core (Rust): the rhythm-tree model, exact timing and the v2 link format; every other component uses it. |
| `crates/rhythm-tree-wasm` | wasm-bindgen bindings that expose the core to the browser. |
| `web/` | Browser client: TypeScript, React, Vite, Vitest. |
| `docs/milestones.md` | The phased plan and what "done" means for each milestone. |
| `docs/rfd/` | Design decisions as RFDs (Requests for Discussion). |
| `docs/analysis/` | Analysis of an existing system (Groove Scribe). |
| `docs/design/` | Screen designs (artboard sources) and the interactive format playground. |
| `docs/reference/rhythmTree.js` | The frozen JavaScript reference: the specification the Rust core is built against. |
| `tools/gen-conformance.mjs` | Regenerates the cross-implementation conformance fixture from the JS reference. |

## Commands

Install [mise](https://mise.jdx.dev), then `mise trust && mise install`; `mise.toml` pins Rust (with the wasm target, rustfmt and clippy), Node 22 and `wasm-pack`.

Run the relevant checks after every change; all must pass before a commit.

| Area | Command |
| --- | --- |
| Rust format | `cargo fmt --all --check` |
| Rust lint | `cargo clippy --workspace --all-targets --all-features -- -D warnings` |
| Rust tests | `cargo test --workspace --all-features` (unit, property, conformance and doc tests) |
| wasm package for the web app | `wasm-pack build crates/rhythm-tree-wasm --target web --out-dir ../../web/src/core/pkg` |
| Web (after the wasm package) | `cd web && npm ci && npm run typecheck && npm test && npm run build` |
| Conformance fixture | `node tools/gen-conformance.mjs` (only to add cases; see below) |

CI (`.github/workflows/ci.yml`) runs the Rust checks, and the web checks after building the wasm package, on pushes to `main` and pull requests, installing toolchains from `mise.toml`.

## The model, and the requirements on the core

Read `docs/design/format-playground.html` (open it in a browser) and the
crate docs in `crates/rhythm-tree/src/lib.rs` before changing the core.

- A bar is a meter, a beat grouping and segments. Every node is an equal split
  into `n` slots; children's weights sum to `n`. There is no tuplet node type:
  the reference's `tupletFor` infers tuplets from written spans, and an explicit
  `n:m` ratio only labels the bracket.
- Time is exact. Onsets are exact rationals in quarter notes. Convert to `f64`
  only at the edge: seconds for audio, pixels for drawing.
- `encode` is canonical: `encode(decode(encode(s))) == encode(s)`.
- `decode` never panics and never fails. Problems come back as `Issue`s and
  whatever could be read is kept. The property tests guard this; keep them
  passing.
- Old Groove Scribe query links (`?TimeSig=…&H=|…|`) must import (milestone 7).
- Hits address leaves in time order, one letter per lane per slot, using
  Groove Scribe's tab letters. Changing a beat's subdivision snaps existing
  hits to the nearest new slot in the editor (milestone 3); the format never
  stores an onset that isn't on the tree.

The JS reference is frozen and specifies the behaviour the core is built to.
The conformance fixture (`crates/rhythm-tree/tests/fixtures/conformance.json`)
records its output for 17 inputs: links, issues, event times and MIDI
resolution. To add a case, add it to `tools/gen-conformance.mjs` and
regenerate. `crates/rhythm-tree/tests/conformance.rs` checks the canonical
link and issues of each case in `CODEC_CASES`; a case is listed once the core
has every feature it uses. Events, MIDI and the legacy flag are checked once
Play, MIDI export and old-link import exist. If the Rust behaviour must change
on purpose, update the fixture's expected values by hand, say why in the
commit message, and redo those edits after regenerating.

## How to work here

- **Plan before code.** For anything beyond a small fix, read the relevant RFD
  and milestone, then propose a plan (files, tests, risks) before editing.
- **Tests first in the core.** New grammar or behaviour gets a unit test, a
  property test where it can, and its fixture case listed in `CODEC_CASES`
  when it affects the format.
- **Keep the core pure.** No I/O, no clocks, no randomness in
  `crates/rhythm-tree`. Services and the client own those.
- **Small commits that say why.** The subject says what changed; the body says
  why and how it was verified.
- **Dependencies need a reason.** Prefer the standard library. Justify each
  new crate or npm package in the commit body.
- **Writing in `docs/rfd/` and `docs/analysis/` belongs to Forrest.** Do not
  write or rewrite the prose in those files. You may add facts, links and
  measurements under a section headed "Notes", suggest an outline, or review a
  draft when asked.
- **Keep the repository about the product.** Docs, comments and commit messages
  describe Tupletstorm and its design, nothing else.

## Current focus

Milestone 1 (notation rendering spike) and milestone 2 (first vertical slice in
the web client). See `docs/milestones.md` for scope and exit criteria.
