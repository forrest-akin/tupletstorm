state: prediscussion
authors: Forrest Akin
discussion:

# RFD 0001: Rhythm tree model

<!-- Skeleton: questions, facts and sources. Write the prose in your own words; delete these comments as you go. -->

## Problem

<!-- What the model must express that a fixed grid can't; who feels the pain. -->

Notes:

- Groove Scribe stores each lane as a fixed-length array per measure (32 slots,
  48 for triplet divisions), with one time signature and one `Div` for the
  whole groove (`js/grooveData.js`, `js/urlSerialization.js` at upstream
  `08d14fb`).
- Requirements: meter changes between bars; a different subdivision per beat;
  per-lane polyrhythm inside a beat; tuplets nested inside tuplets; tuplets
  that cross beat lines.

## Proposal

<!-- The structure, in your words. Use the nested-tuplet bar as the worked example. -->

Notes:

- Bar = meter + beat grouping + segments. Segment weights count beat groups.
- Node = equal split into `n` slots. Children fill slots; weights sum to `n`.
  Absent children means `n` leaves.
- No tuplet node type. `tuplet_for(written_span, n, m)` decides; `m` is display only.
- Time = exact rationals in quarter notes. Example: the nested quintuplet in
  `4/4:3:2(2,2,5:4)@2,…` starts at 4/3 and its notes are 2/15 long.
- Lane overrides graft a subtree for one lane (hi-hat 5 over 4).
- Hits are one letter per leaf in time order, per lane.

## Alternatives considered

<!-- Why not these? -->

- Tick grid at a fixed PPQ (MIDI style).
- Floating-point onsets per note.
- A separate tuplet node type, as in MusicXML's `<time-modification>`.
- Notes stored by onset with the tree only as a view.

## Consequences

<!-- What gets easier, what gets harder (for example, stable identity for collaborative edits). -->

- Editing a subdivision changes leaf indices; the editor snaps hits.
- Concurrent edits need stable addressing or op transformation (see RFD 0005).

## Sources

- `crates/rhythm-tree/src/` — the implementation.
- `docs/design/format-playground.html` — interactive walkthrough.
- Upstream Groove Scribe: https://github.com/montulli/GrooveScribe
