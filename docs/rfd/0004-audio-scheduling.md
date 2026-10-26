state: ideation
authors: Forrest Akin
discussion:

# RFD 0004: Audio scheduling

## Question

How do we play a groove so that edits are heard within one scheduling window,
timing stays exact over long loops, and samples load without stalling playback?

## Notes

- `compile_events` returns events in seconds, computed from exact rationals.
- Groove Scribe builds a MIDI file and reloads it at the next loop when notes
  change (`midiNoteHasChanged` → `loadMidiDataEvent` in `js/groove_utils.js`),
  at 128 ticks per quarter (`js/jsmidgen.js`), so 16th-note triplets fall on
  10.666 ticks (`js/midiFile.js`).
- Standard approach to evaluate: a lookahead scheduler (a timer every ~25 ms
  queues `AudioBufferSourceNode`s ~100 ms ahead on the `AudioContext` clock).
- Sample loading: lazy, cached, and concurrent requests for the same sample
  share one in-flight fetch.

## Open questions

- Swing: apply in the scheduler to binary splits only, as Groove Scribe does?
- Metronome: generated clicks from the beat grouping, or sampled?
- Where does the scheduler live: main thread, AudioWorklet, or a Worker?

## Sources

- Chris Wilson, "A Tale of Two Clocks": https://web.dev/articles/audio-scheduling
- `crates/rhythm-tree/src/play.rs`
