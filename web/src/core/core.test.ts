import { readFileSync } from 'node:fs';
import { expect, test } from 'vitest';
import { initSync } from './pkg/rhythm_tree_wasm.js';
import { core } from './core';

initSync({ module: readFileSync(new URL('./pkg/rhythm_tree_wasm_bg.wasm', import.meta.url)) });

const { openLink, view, setHit } = core;

const { cases } = JSON.parse(
  readFileSync(new URL('../../../crates/rhythm-tree/tests/fixtures/conformance.json', import.meta.url), 'utf8'),
) as { cases: { input: string; encoded: string }[] };

const KIT = ['cr', 'rd', 'hh', 'sk', 'cb', 'sn', 't1', 't2', 't3', 't4', 'kk', 'hf', 'mt', 'st'];

test('opens a v2 link through the wasm package', () => {
  const link = 'https://x.test/#v=2&b=4/4:4;7/8:4,4,6';

  expect(openLink(link).issues).toEqual([]);
  const { bars } = view(link, []);
  expect(bars.map((b) => b.meter)).toEqual(['4/4', '7/8']);
  expect(bars.map((b) => b.leaves.length)).toEqual([16, 14]);
  expect(bars[1].length).toEqual({ num: 7, den: 2 });
  expect(bars[1].leaves[13]).toEqual({ segment: 2, start: { num: 13, den: 4 }, span: { num: 1, den: 4 } });
});

test('reports a link with no groove', () => {
  expect(openLink('https://example.com/').issues).toEqual([
    { level: 'warn', msg: 'no v=2 and no Groove Scribe fields; starting empty' },
  ]);
});

test("opens every kit lane, then the link's custom lanes", () => {
  const opened = openLink(cases[11].input);
  expect(opened.lanes).toEqual(KIT);
  expect(opened.issues).toEqual([
    { level: 'warn', msg: 'kk bar 0: 3 hits for 16 slots; padded' },
    { level: 'warn', msg: 'sn bar 0: 20 hits for 16 slots; cut' },
  ]);
  const sn = view(cases[11].input, ['sn', 'kk']).lanes[0];
  expect(sn).toMatchObject({ id: 'sn', name: 'Snare', letters: '-oOgxfdb' });
  expect(sn.bars[0].hits).toBe('o---o---o---o---');
  expect(openLink('#v=2&b=4/4:4&zz9=x').lanes).toEqual([...KIT, 'zz9']);
});

test('reads the title and tempo', () => {
  const { title, tempo } = view(cases[0].input, []);
  expect({ title, tempo }).toEqual({ title: 'Bars 1–2', tempo: 96 });
});

test('canonicalises the opened link', () => {
  expect(openLink(cases[0].input).link).toBe(cases[0].encoded);
});

test('rewrites the link when a hit changes', () => {
  const { link } = openLink(cases[0].input);
  expect(setHit(link, 'hh', 0, 1, 'x')).toBe(link.replace('hh=x-x-x-x-x-x-7;', 'hh=xxx-x-x-x-x-7;'));
  expect(() => setHit(link, 'hh', 9, 0, 'x')).toThrow("bar 9 doesn't exist");
});

test('views a lane whose last hit was cleared', () => {
  const l = setHit(openLink('#v=2&b=4/4:4&hh=x').link, 'hh', 0, 0, '-');
  expect(l).not.toContain('hh=');
  expect(view(l, ['hh']).lanes[0].bars[0].hits).toBe('-'.repeat(16));
});

test('edits a custom lane whose last hit was cleared', () => {
  const l = setHit(openLink('#v=2&b=4/4:4&zz9=x').link, 'zz9', 0, 0, '-');
  expect(setHit(l, 'zz9', 0, 0, 'x')).toContain('zz9=x-15');
});
