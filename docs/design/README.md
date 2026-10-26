# Design references

| File | What it is |
| --- | --- |
| `format-playground.html` | Interactive walkthrough of the rhythm-tree model and link format. Open it in a browser: edit a link and watch the tree, lanes and timings update, or click lanes and watch the link change. It runs the frozen JS reference. |
| `artboards/*.dc.html` | Source markup of the five screen designs. They were authored for a design canvas and won't render on their own (they expect that canvas's `support.js` runtime), but the layout, copy, colours and interaction logic are all readable here. |

## The screens

All desktop screens are fluid pages designed at 1440 px wide; the practice
view is a 390 × 844 phone layout.

1. **Editor** (`Main.dc.html`). A top bar has Edit / Practice / Print views,
   undo, redo and Share. Below it, the transport: play, stop, loop, click,
   count-in, tempo with tap, and a position readout. The main area lines rows
   up on one time axis: bar ribbon (meter and grouping), notation, subdivision
   chips per beat, a counting row, then one lane per kit piece, where clicking
   a cell cycles hit, accent and ghost. The inspector shows the selected beat:
   subdivision options (changing one snaps existing hits), the bar's meter,
   tuplet details and keyboard shortcuts.
2. **Mixed meters** (`Meter.dc.html`). The arrangement shows bars sized by
   duration, with the meter printed only where it changes and pulse dots by
   beat group. Panels: time signature (stepper, beat unit, common meters),
   beat grouping (presets such as 2+2+3, 2+3+2, 3+2+2), and apply scope ("this
   bar" or "until the next change"), the tempo relation across the barline
   (♪ = ♪ or ♩ = ♩.), and what happens when a bar shrinks.
3. **Per-beat subdivisions** (`Subdivision.dc.html`). A one-bar lens. Each
   beat's menu sets its split for all lanes or for one lane (a lane override).
   Hits sit on their onset lines. An onset map shows where every lane's hits
   land, and a timing table lists each slot's onset in milliseconds.
4. **Nested tuplets** (`Tuplets.dc.html`). Bracket rows, one per nesting level,
   above the lanes; brackets can cross beat lines. Beside them: a rhythm-tree
   outline, and details for the selected tuplet (ratio chain, written value,
   milliseconds per note, how the number displays).
5. **Practice player** (`Phone.dc.html`). Notation for the current bar with a
   playhead, a bar loop, big transport controls, tempo with a speed trainer
   (+4 every 4 loops up to a target), and muting parts to play them yourself.

## Visual language

- Dark "studio console" ground: graphite `#121417`, panels `#1A1D22`, lines `#2E333B`.
- Text `#ECE9E2`, secondary `#A9AEB8`, tertiary `#8A909B`.
- Amber `#F2A541` for hits, the playhead and primary actions; blue `#6CB4EE`
  for selection and tuplets. Accents are drawn larger with a `>` mark, ghost
  notes as rings, so state never depends on colour alone.
- Notation sits on paper `#F3EFE6` in ink `#17181B`.
- Type: IBM Plex Sans for the interface, IBM Plex Mono for numbers, meters and link text.
