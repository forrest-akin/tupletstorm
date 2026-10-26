# Tupletstorm

Drum and percussion notation for the grooves other editors make hard: meter
changes from bar to bar, a different subdivision on every beat, a hi-hat in
five against a kick in four, tuplets inside tuplets. Write it, hear it, and
share it as a link.

**Status:** see [`docs/milestones.md`](docs/milestones.md) for what works.

## How it works

Every bar is a tree of equal splits. A node divides the time it is given into
`n` slots, and its children fill those slots, so

```
4/4:3:2(2,2,5:4)@2,5:4(1,1,1,3:2@2)@2
```

is one 4/4 bar: a quarter-note triplet over beats 1–2 whose third note holds a
quintuplet, then an eighth-note quintuplet over beats 3–4 whose last two notes
hold a triplet. Time is kept as exact fractions of a quarter note (that
quintuplet's second note starts at 22/15), so nested tuplets never drift, and
whether a split is drawn as a tuplet is worked out from its length rather than
stored.

A whole groove fits in a readable link fragment:

```
#v=2&q=96&b=4/4:4,4,4,6;7/8:4,4,6&hh=x-x-x-x-x-x-7;--x-x-x-x-x-x-&sn=-4o--g-4Ooo-3;-4o-7o-&kk=o-o-5oo-8;o-7o-o-3
```

Links from Groove Scribe are to open too (milestone 7).
[`docs/design/format-playground.html`](docs/design/format-playground.html) is
an interactive tour of both.

## Layout

```
docs/    milestones, RFDs, analysis, designs, the format reference
tools/   conformance fixture generation
```

## Development

Install [mise](https://mise.jdx.dev), then run `mise trust && mise install`.
`mise.toml` pins Node 22, which runs `tools/gen-conformance.mjs`.

## Licence

[Mozilla Public License 2.0](LICENSE).
