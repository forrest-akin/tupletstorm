# Groove Scribe under edit and playback

<!--
Skeleton for an analysis of a system you didn't build. Measure first, then
explain. Write the prose yourself; keep the facts below as your starting
evidence, and replace each "to measure" with numbers and how you got them.
-->

## Question

<!-- One or two sentences: what behaviour are you explaining, and why does it matter to a drummer using it? -->

## System under study

- Groove Scribe, upstream https://github.com/montulli/GrooveScribe at commit
  `08d14fb` (2026-10-03). Served locally with `npm run serve`.
- Render pipeline: URL → `grooveData` → ABC → SVG (abc2svg) and → MIDI file
  (jsmidgen) → MIDI.js soundfont playback.

## Observations from the source (to confirm by measurement)

| # | Observation | Where |
| --- | --- | --- |
| 1 | Every note change regenerates the ABC for the whole groove, re-renders it through abc2svg and replaces the SVG with `innerHTML`. | `updateSheetMusic`, `displayNewSVG` in `js/groove_writer.js` |
| 2 | The grid's state is read back from each cell's rendered colour rather than from a model. | `js/gridState.js` |
| 3 | After a change during playback, a new MIDI file is generated and loaded at the next loop. | `midiNoteHasChanged`, `loadMidiDataEvent` in `js/groove_utils.js` |
| 4 | MIDI files use 128 ticks per quarter; 48th-note triplets are given 10.666 ticks. | `js/jsmidgen.js`, `js/midiFile.js` |
| 5 | One time signature and one division per groove; at most 10 measures; triplet divisions only fit 2/4 and 4/4. | `js/urlSerialization.js`, `js/constants.js`, `js/musicMath.js` |

## Measurements to take

| # | Metric | Method | Result |
| --- | --- | --- | --- |
| A | Time from a cell click to the updated SVG painted, for 1, 4 and 10 measures, with and without toms and stickings | Chrome Performance panel, or a Playwright script with `performance.mark` around the click handler | to measure |
| B | Share of that time in ABC generation vs abc2svg vs DOM replacement | Performance panel call tree | to measure |
| C | Worst-case delay from an edit to hearing it during playback, by tempo and measure count | Reasoning from observation 3, then confirmed by recording output | to measure |
| D | Timing error from tick rounding for 16th-note triplets at 128 PPQ, per note and accumulated over a loop | Compute expected vs rounded onsets; inspect the generated MIDI file | to measure |
| E | URL length by measure count and lanes shown | Generate links for 1–10 measures | to measure |

## Explanation

<!-- Tie each measured cost back to a design decision. Which costs grow with groove length? Which are constant? -->

## What would fix it

<!-- Concrete changes, each linked to a measurement. Point to RFDs 0001, 0003 and 0004 where Tupletstorm does it differently. -->

## Notes

<!-- Scratch space for raw numbers, screenshots and links. -->
