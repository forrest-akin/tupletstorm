# Milestones

Tupletstorm grows in phases. Each phase ships something people can use, and each
later phase adds one kind of distributed-systems component on top of the same
core library.

Status: **phase 0 done**. Phase 1 is in progress: in milestone 2, opening a v2
link, drawing its bars and lanes, and cycling a cell to rewrite the link work;
Play and sample loading wait on RFD 0004 (audio scheduling).

## Phase 0: design and format references (done)

- Design references (`docs/design/`) and RFD skeletons (`docs/rfd/`).
- The format reference (`docs/reference/rhythmTree.js`) and its conformance
  fixture (`crates/rhythm-tree/tests/fixtures/conformance.json`).
- Node, Rust and wasm-pack pinned in `mise.toml`.

## Phase 1: local-first editor (static site)

Everything runs in the browser and a groove is its link. No server.

The core (`crates/rhythm-tree`) and its browser bindings are built as
milestones call them: the v2 link codec and bar walk in 2, the event compiler
with Play, tuplet inference with notation (1 and 3), MIDI resolution in 6, and
old-link import in 7.

| # | Milestone | Exit criteria |
| --- | --- | --- |
| 1 | **Notation rendering spike** (RFD 0003) | One bar from each example groove drawn on a five-line drum staff aligned to the grid's time axis, including the nested-tuplet bar. Decision recorded: own renderer or a library, with evidence. |
| 2 | **Vertical slice** | The wasm package loads in the client. Opening a v2 link draws the bar grid from the bar's leaves, clicking a cell cycles its letter and rewrites the link (debounced), and Play schedules `compileEvents` through Web Audio. Drum samples load lazily, and concurrent requests for the same sample share one fetch. |
| 3 | **Editor screen** | Matches `docs/design/artboards/Main.dc.html`: transport, bar ribbon, notation, subdivision chips, lanes, inspector. Changing a beat's subdivision snaps hits. Undo and redo. |
| 4 | **Meters, subdivision lens, tuplet editor** | The three feature screens in `docs/design/artboards/`, including lane overrides and nesting tuplets from the UI. |
| 5 | **Practice view** | The phone layout: loop, tempo trainer, muting parts. |
| 6 | **Exports in the browser** | MIDI at `midiResolution` PPQ; SVG of the score. |
| 7 | **Ship it** | Deployed to a static host with a custom domain; old Groove Scribe links open; decode cost on hostile links measured and bounded. |

Parallel track: the Groove Scribe analysis (`docs/analysis/groove-scribe.md`).

## Phase 2: accounts, library and sharing

| # | Milestone | Exit criteria |
| --- | --- | --- |
| 8 | **API service** | A Rust HTTP service with an OpenAPI spec and a generated TypeScript client the web app uses. Grooves saved by revision; short links. |
| 9 | **Identity and permissions** | Sign-in, workspaces, and sharing (view, comment, edit) with a written authorization model and tests for it. |
| 10 | **Operations baseline** | Deployed with CI/CD, structured logs, metrics, an uptime SLO and a dashboard; backups restore cleanly. |

## Phase 3: real-time collaboration

| # | Milestone | Exit criteria |
| --- | --- | --- |
| 11 | **Sync service** | Several people edit one groove over WebSockets. The server orders operations and validates them with the core. The operation log is durable, and clients reconnect and catch up. |
| 12 | **Control plane** | Document sessions are placed on sync nodes, moved when a node drains, and recovered when one dies. A load test and a failure-injection run, written up. |

## Phase 4: durable exports

| # | Milestone | Exit criteria |
| --- | --- | --- |
| 13 | **Export pipeline** | PDF and audio rendering run as retried, idempotent background jobs; results land in object storage, with progress shown in the client. |

## Phase 5: open questions

Pricing, and what would be worth paying for, are decided after people use
phase 1. See RFD 0006.
