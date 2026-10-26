import { readFileSync } from 'node:fs';
import { expect, test } from 'vitest';
import { initSync } from './pkg/rhythm_tree_wasm.js';
import { core } from './core';

initSync({ module: readFileSync(new URL('./pkg/rhythm_tree_wasm_bg.wasm', import.meta.url)) });

const { openLink, view } = core;

const { cases } = JSON.parse(
  readFileSync(new URL('../../../crates/rhythm-tree/tests/fixtures/conformance.json', import.meta.url), 'utf8'),
) as { cases: { input: string }[] };

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
