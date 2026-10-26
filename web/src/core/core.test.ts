import { readFileSync } from 'node:fs';
import { expect, test } from 'vitest';
import { initSync } from './pkg/rhythm_tree_wasm.js';
import { core } from './core';

initSync({ module: readFileSync(new URL('./pkg/rhythm_tree_wasm_bg.wasm', import.meta.url)) });

const { openLink, view } = core;

test('opens a v2 link through the wasm package', () => {
  const link = 'https://x.test/#v=2&b=4/4:4;7/8:4,4,6';

  expect(openLink(link).issues).toEqual([]);
  const { bars } = view(link);
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
