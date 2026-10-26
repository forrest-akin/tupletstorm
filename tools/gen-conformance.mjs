// Regenerates crates/rhythm-tree/tests/fixtures/conformance.json from the frozen
// JavaScript reference. Add a case to `inputs`, then run `node tools/gen-conformance.mjs`.
import * as R from '../docs/reference/rhythmTree.js';
import fs from 'fs';

const s1 = R.emptyScore(); s1.title = 'Bars 1–2'; s1.tempo = 96;
s1.bars = [R.parseBar('4/4:4,4,4,6').bar, R.parseBar('7/8:4,4,6').bar];
s1.hits = { cr: ['-'.repeat(18), 'x' + '-'.repeat(13)], hh: ['x-x-x-x-x-x-------', '--x-x-x-x-x-x-'],
  sn: ['----o--g----Ooo---', '----o-------o-'], t1: ['---------------oo-', '-'.repeat(14)],
  t4: ['-'.repeat(17) + 'o', '-'.repeat(14)], kk: ['o-o-----oo--------', 'o-------o-o---'] };
const s2 = R.emptyScore(); s2.title = 'Five over four'; s2.tempo = 96;
s2.bars = [R.parseBar('4/4:4,4,4,3').bar];
s2.overrides = [{ lane: 'hh', bar: 0, path: [2], node: R.parseNode('5') }];
s2.hits = { hh: ['xxxxxxxxxxxxx---'], sn: ['----o-----o-Ooo'], kk: ['o--o--o-o--oo--'] };
const s3 = R.emptyScore(); s3.title = 'Nested tuplets'; s3.tempo = 90;
s3.bars = [R.parseBar('4/4:3:2(2,2,5:4)@2,5:4(1,1,1,3:2@2)@2').bar];
s3.hits = { sn: ['o'.repeat(15)], kk: ['o--------o-----'] };
const s4 = R.emptyScore(); s4.title = 'Meter tour'; s4.tempo = 120;
s4.bars = ['4/4:4', '7/8:4,4,6', '5/8~3-2:6,4', '6/8:6', '6/8:6'].map((b) => R.parseBar(b).bar);
s4.tempoChanges = [{ bar: 3, unit: R.rat(3, 2), old: R.rat(1) }];
s4.hits = { hh: ['x-x-x-x-x-x-x-x-', 'x-x-x-x-x-x-x-', 'x-x-x-x-x-', 'x--x--x--x--', 'x--x--x--x--'],
  sn: ['----o-------o---', '----o-------o-', '------o---', '------o-----', '------o--o-o'],
  kk: ['o-------o-o-----', 'o-------o-o---', 'o-----o---', 'o-----------', 'o-----------'] };

const inputs = [
  '#' + R.encode(s1), '#' + R.encode(s2), '#' + R.encode(s3), '#' + R.encode(s4),
  'https://www.mikeslessons.com/groove/?TimeSig=4/4&Div=16&Title=Basic%20Rock&Tempo=90&Measures=2&H=|x-x-x-x-x-x-x-x-|c-x-x-x-x-x-x-x-|&S=|----O-------O---|----O-------O--g|&K=|o-------o-o-----|X-------o-------|',
  '?TimeSig=4/4&Div=12&H=|x-xx-xx-xx-x|',
  '?TimeSig=6/8&Div=8&S=|o-o-o-o-o-o-|',
  '?timesig=7/8&div=16&measures=1&h=|x-x-x-x-x-x-x-|&k=|X-----o-------|&s=|----O---q-----|&Title=Odd+Time%20Groove&tempo=500',
  '?TimeSig=4/4&Div=8&Measures=2&H=|x-x-x-x-x-x-x-x-|x-x-x-x-x-x-x-x-|',
  '?TimeSig=7/8&Div=12&H=|xxxxxxxxxxx|&Swing=30',
  '#v=2&q=96&b=4/4:5:4(1,1,1);4/4:4&sn=ozz-;o---o---o---o---',
  '#v=2&b=4/4:4&kk=o-o&sn=o---o---o---o---o---',
  '#v=2&q=abc&b=4/4:4;7/8:4,4,6&ov=hh@0.9!5;hh@3.0!3;XX@0.1!3;sn@1.2!3:2&tc=1:3/2~1;x:1~1;1:1/0~1&Weird=1&zz9=x-x&hh=x-x-x-x-x-x-x-x-;.',
  'https://example.com/?v=2&q=120&sw=20&b=6/8:6*2&hh=x--x--x--x--;.',
  'https://example.com/',
  '#v=2&t=A%20%26%20B&a=F.%20Akin&c=%E2%99%A9%3D96&q=92.5&b=4/4:4,4,4,6:4;3/4:3(1@2,1);12/16~3-3-3-3:3*2&cr=x-15;-9&hh=x-x-x-x-x-x-x-x-;x--x--;;x-xx-xx-xx-x',
  '#v=2&b=4/4:4*3;5/4~3-2:4@3,4@2&ov=hh@1.0.2!3;hh@1.0!5:4;kk@3.1!7&hh=x-x-x-x-x-x-x-x-;.;.;xxxxx--------&kk=;;;o-o-o-o-o',
];

const cases = inputs.map((input) => {
  const d = R.decode(input);
  const c = R.compileEvents(d.score);
  return {
    input,
    encoded: R.encode(d.score),
    legacy: d.legacy,
    issues: d.issues.map((i) => [i.level, i.msg]),
    duration: c.duration,
    events: c.events.map((e) => [e.t, e.lane, e.art, e.bar, e.leaf]),
    midi: R.midiResolution(d.score),
  };
});
fs.writeFileSync(new URL('../crates/rhythm-tree/tests/fixtures/conformance.json', import.meta.url), JSON.stringify({ generatedBy: 'docs/reference/rhythmTree.js', cases }, null, 1) + '\n');
for (const c of cases) console.log(c.issues.length, 'issues |', c.encoded.slice(0, 100));
