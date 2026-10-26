state: prediscussion
authors: Forrest Akin
discussion:

# RFD 0002: Link format v2

<!-- Skeleton: questions, facts and sources. Write the prose yourself. -->

## Goals

<!-- Readable, short, canonical, versioned, safe to decode from anyone, backward compatible. Rank them. -->

## Format

Notes:

- Lives in the fragment (`#v=2&…`): never sent to a server; the client
  rewrites it on every edit with `history.replaceState`.
- Fields: `v`, `t`, `a`, `c`, `q` (quarter notes per minute), `sw`, `b` (bars),
  `tc` (tempo changes), `ov` (lane overrides), then one field per lane.
- Grammar (also in `docs/design/format-playground.html`):

```
node  := INT [ ':' INT ] [ '(' node { ',' node } ')' ] [ '@' INT ]
bar   := INT '/' INT [ '~' INT { '-' INT } ] ':' node { ',' node } [ '*' INT ]
hits  := { letter | '-' | '-' INT } | '.'
ov    := lane '@' bar { '.' index } '!' node
tc    := bar ':' frac '~' frac
```

- Every character used is legal unescaped in a fragment (RFC 3986 §3.5);
  only free text is percent-encoded.
- Encoding is canonical; decoding never fails (problems become issues).

## Compatibility

Notes:

- Links without `v=2` that carry Groove Scribe fields are imported, mirroring
  its parser: case-insensitive names, tab-line rescaling, tempo 20–400,
  10 measures maximum.
- Example: a two-bar rock groove is 172 characters as a Groove Scribe query
  and 129 as a v2 fragment.

## Open questions

- A compressed form for very long grooves (`#z=` + deflate + base64url)?
- 0-based bar indices in `ov` and `tc`: keep, or switch to 1-based for readability?
- How do saved grooves (phase 2) relate to links: the link as the document, or
  a pointer to a revision?

## Sources

- `crates/rhythm-tree/src/codec.rs`, `crates/rhythm-tree/src/legacy.rs`
- Groove Scribe `js/urlSerialization.js`, `js/noteArrays.js` at `08d14fb`
- RFC 3986: https://www.rfc-editor.org/rfc/rfc3986
