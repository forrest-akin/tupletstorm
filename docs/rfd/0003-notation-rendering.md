state: ideation
authors: Forrest Akin
discussion:

# RFD 0003: Notation rendering

<!-- Decide after the milestone 1 spike. Record the evidence, not just the choice. -->

## Question

How do we draw a drum staff from the rhythm tree: in the editor (aligned to
the grid's time axis, redrawn per bar as you edit) and for print or export
(conventional spacing)?

## Options to evaluate

- **Own renderer** for the editor: drum staff positions are a small fixed set;
  `barNodes` already yields onsets, written values and tuplet brackets.
  Proportional spacing comes for free.
- **abc2svg** (what Groove Scribe uses): check nested tuplets and per-bar rendering.
- **VexFlow**: check percussion noteheads, nested tuplets, licence.
- **Verovio** (MEI input, WebAssembly): check bundle size, nested tuplets, licence.

## Spike checklist

- [ ] The nested-tuplet bar renders correctly in each candidate.
- [ ] Time to redraw one bar after an edit (target: well under one frame).
- [ ] Bundle size added to the client.
- [ ] Licence compatible with MPL-2.0.

## Notes

- Groove Scribe regenerates ABC for the whole groove and re-renders all of it
  on every edit (`updateSheetMusic` in `js/groove_writer.js` at `08d14fb`).
- The editor design aligns notation to the grid (`docs/design/artboards/Main.dc.html`).

## Decision

<!-- Fill in after the spike. -->
