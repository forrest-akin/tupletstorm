// rhythmTree.js — proposed v2 rhythm model + URL codec for Groove Scribe.
//
// A bar is a meter, a beat grouping, and a list of segments. Each segment is a
// node, and every node is one thing: an n-way equal split of the time it
// occupies — either into n leaves, or into child nodes whose weights (the
// number of slots each fills) sum to n.
//
// There is no tuplet node type. Whether a split engraves as a tuplet falls out
// of its *written* span (see tupletFor): 3 in a quarter is a triplet, 3 in a
// dotted quarter is plain 8ths. An optional ratio (n:m) only overrides how the
// bracket is labelled; it never changes timing.
//
// All timing is exact rational arithmetic in quarter notes, so nested tuplets
// never drift, and MIDI tick resolution can be chosen to make every onset an
// integer.
//
// Pure module: no DOM, no GrooveUtils.

export const MAX_SLOTS = 64;
export const MAX_DEPTH = 6;
export const MAX_BARS = 512;
export const MAX_LEAVES_PER_BAR = 512;

/* ------------------------------------------------------------------ */
/* Rationals                                                           */
/* ------------------------------------------------------------------ */

/** @typedef {[number, number]} Rat  [numerator, denominator]; denominator > 0, reduced. */

function gcd(a, b) {
  a = Math.abs(a);
  b = Math.abs(b);
  while (b) [a, b] = [b, a % b];
  return a;
}

function lcm(a, b) {
  return (a / gcd(a, b)) * b;
}

/**
 * @param {number} n
 * @param {number} [d]
 * @returns {Rat}
 */
export function rat(n, d = 1) {
  if (!Number.isInteger(n) || !Number.isInteger(d) || d === 0) {
    throw new RangeError(`bad rational ${n}/${d}`);
  }
  if (d < 0) {
    n = -n;
    d = -d;
  }
  const g = gcd(n, d) || 1;
  return [n / g, d / g];
}

/** @param {Rat} a @param {Rat} b @returns {Rat} */
export function radd(a, b) {
  const l = lcm(a[1], b[1]);
  return rat(a[0] * (l / a[1]) + b[0] * (l / b[1]), l);
}

/** @param {Rat} a @param {Rat} b @returns {Rat} */
export function rmul(a, b) {
  const g1 = gcd(a[0], b[1]) || 1;
  const g2 = gcd(b[0], a[1]) || 1;
  return rat((a[0] / g1) * (b[0] / g2), (a[1] / g2) * (b[1] / g1));
}

/** @param {Rat} a @param {Rat} b @returns {Rat} */
export function rdiv(a, b) {
  if (b[0] === 0) throw new RangeError('divide by zero');
  return rmul(a, [b[1], b[0]]);
}

/** Negative, zero or positive as a <, =, > b. @param {Rat} a @param {Rat} b */
export function rcmp(a, b) {
  return a[0] * b[1] - b[0] * a[1];
}

/** @param {Rat} a */
export const rnum = (a) => a[0] / a[1];

/** @param {Rat} a */
export const rstr = (a) => (a[1] === 1 ? String(a[0]) : `${a[0]}/${a[1]}`);

/** @param {string} s @returns {Rat} */
function parseRat(s) {
  const m = /^(\d+)(?:\/(\d+))?$/.exec(s);
  if (!m) throw new SyntaxError(`bad fraction '${s}'`);
  return rat(Number(m[1]), m[2] ? Number(m[2]) : 1);
}

function isPow2Int(x) {
  return x > 0 && 2 ** Math.round(Math.log2(x)) === x;
}

/** True for 1, 2, 4 … and 1/2, 1/4 … @param {Rat} r */
function isPow2(r) {
  return (r[0] === 1 && isPow2Int(r[1])) || (r[1] === 1 && isPow2Int(r[0]));
}

/** Smallest 2^k (k may be negative) that is >= r. @param {Rat} r @returns {Rat} */
function pow2AtLeast(r) {
  let p = rat(1);
  while (rcmp(p, r) < 0) p = rmul(p, [2, 1]);
  while (rcmp(rmul(p, [1, 2]), r) >= 0) p = rmul(p, [1, 2]);
  return p;
}

/* ------------------------------------------------------------------ */
/* Model                                                               */
/* ------------------------------------------------------------------ */

/**
 * One node of a bar's rhythm tree.
 *
 * @typedef {Object} RNode
 * @property {number} n        Slots: equal parts this node splits its span into.
 * @property {number} w        Weight: how many of the parent's slots it fills (default 1).
 *                             At bar level, weight counts beat groups.
 * @property {number} [m]      Display ratio n:m for the tuplet bracket (engraving only).
 * @property {RNode[]} [kids]  Children filling the n slots; weights sum to n. Absent = n leaves.
 */

/**
 * @typedef {Object} Bar
 * @property {number} num       Meter numerator.
 * @property {number} den       Meter denominator: 1, 2, 4, 8, 16 or 32.
 * @property {number[]} groups  Beat grouping in 1/den units, summing to num (7/8 → [2, 2, 3]).
 * @property {RNode[]} segs     Segments; their weights count beat groups and sum to groups.length.
 */

/**
 * A lane that replaces one subtree with its own (polyrhythm: hi-hat in 5 while
 * the kick stays in 4). The replacement keeps the weight of what it replaces.
 *
 * @typedef {Object} Override
 * @property {string} lane
 * @property {number} bar     0-based bar index.
 * @property {number[]} path  [segment, child, child …], 0-based.
 * @property {RNode} node
 */

/**
 * Metric modulation at the start of a bar: a `unit` note (in quarters) now
 * lasts as long as an `old` note did. ♩. = ♩ is { unit: 3/2, old: 1 }.
 *
 * @typedef {Object} TempoChange
 * @property {number} bar
 * @property {Rat} unit
 * @property {Rat} old
 */

/**
 * @typedef {Object} Score
 * @property {string} title
 * @property {string} author
 * @property {string} comments
 * @property {number} tempo                 Quarter notes per minute at bar 1.
 * @property {number} swing                 0–100, applied at playback as today.
 * @property {Bar[]} bars
 * @property {Record<string, string[]>} hits  lane → one string per bar, one char per leaf ('-' = rest).
 * @property {Override[]} overrides
 * @property {TempoChange[]} tempoChanges
 */

/**
 * The default kit. One articulation per lane per leaf; simultaneous pieces are
 * separate lanes (Groove Scribe's single H lane can't hold a crash and a
 * hi-hat at once; here they're independent). Characters are Groove Scribe's
 * own tab letters, so legacy import is a re-mapping, not a translation.
 */
export const LANES = {
  cr: { name: 'Crash', chars: 'xX' },
  rd: { name: 'Ride', chars: 'xXb' },
  hh: { name: 'Hi-hat', chars: 'xXo+' },
  sk: { name: 'Stacker', chars: 'x' },
  cb: { name: 'Cowbell', chars: 'x' },
  sn: { name: 'Snare', chars: 'oOgxfdb' },
  t1: { name: 'High tom', chars: 'oO' },
  t2: { name: 'Mid tom', chars: 'oO' },
  t3: { name: 'Low tom', chars: 'oO' },
  t4: { name: 'Floor tom', chars: 'oO' },
  kk: { name: 'Kick', chars: 'oO' },
  hf: { name: 'Hi-hat foot', chars: 'x' },
  mt: { name: 'Metronome', chars: 'nN' },
  st: { name: 'Sticking', chars: 'RLBc' },
};

export const LANE_ORDER = Object.keys(LANES);

const META_KEYS = new Set(['v', 't', 'a', 'c', 'q', 'sw', 'b', 'tc', 'ov']);

/**
 * Default beat grouping: quarter-note meters beat by beat; 8th/16th meters in
 * 3s when divisible by 3, otherwise 2s ending in a 3 (7/8 → 2+2+3).
 *
 * @param {number} num
 * @param {number} den
 * @returns {number[]}
 */
export function defaultGroups(num, den) {
  if (den <= 4) return Array(num).fill(1);
  if (num % 3 === 0) return Array(num / 3).fill(3);
  if (num < 4) return [num];
  const g = [];
  let r = num;
  while (r > 3) {
    g.push(2);
    r -= 2;
  }
  g.push(r);
  return g;
}

/**
 * A bar in 16ths with the default grouping.
 *
 * @param {number} num
 * @param {number} den
 * @returns {Bar}
 */
export function defaultBar(num, den) {
  const groups = defaultGroups(num, den);
  return {
    num,
    den,
    groups,
    segs: groups.map((g) => ({ n: Math.max(1, Math.round((g * 16) / den)), w: 1 })),
  };
}

/** @returns {Score} */
export function emptyScore() {
  return {
    title: '',
    author: '',
    comments: '',
    tempo: 80,
    swing: 0,
    bars: [defaultBar(4, 4)],
    hits: {},
    overrides: [],
    tempoChanges: [],
  };
}

/** Bar length in quarter notes. @param {Bar} bar @returns {Rat} */
export function barLength(bar) {
  return rat(bar.num * 4, bar.den);
}

/** @param {RNode} node @returns {RNode} */
function cloneNode(node) {
  /** @type {RNode} */
  const c = { n: node.n, w: node.w };
  if (node.m) c.m = node.m;
  if (node.kids) c.kids = node.kids.map(cloneNode);
  return c;
}

/* ------------------------------------------------------------------ */
/* Engraving: is this split a tuplet?                                  */
/* ------------------------------------------------------------------ */

/**
 * How a split engraves. `writtenSpan` is the node's span in *written* quarters:
 * inside a 3:2, a triplet quarter is written as a quarter, so its children are
 * judged against a written span of 1.
 *
 * Rule: if span/n is a plain note value, it's not a tuplet. Otherwise the slot
 * is written as the smallest power-of-two value >= its real length (falling
 * back one size smaller when that doesn't tile the span), and m = how many of
 * those fill the span. 3 in ♩ → 3:2 ♪; 5 in ♩ → 5:4 ♬; 2 in ♩. → 2:3 ♪.
 *
 * @param {Rat} writtenSpan
 * @param {number} n
 * @param {number} [m]  Explicit ratio from the URL; overrides the inference.
 * @returns {{n: number, m: (number|null), w: Rat} | null}
 *   null = plain note values. w = written value of one slot. m = null means
 *   show "n" alone (irregular span).
 */
export function tupletFor(writtenSpan, n, m) {
  if (n === 1) return null;
  if (m) return m === n ? null : { n, m, w: rdiv(writtenSpan, rat(m)) };
  const d = rdiv(writtenSpan, rat(n));
  if (isPow2(d)) return null;
  let w = pow2AtLeast(d);
  let k = rdiv(writtenSpan, w);
  if (k[1] !== 1) {
    w = rmul(w, rat(1, 2));
    k = rdiv(writtenSpan, w);
  }
  return k[1] === 1 ? { n, m: k[0], w } : { n, m: null, w: d };
}

/* ------------------------------------------------------------------ */
/* Tree walking                                                        */
/* ------------------------------------------------------------------ */

/**
 * @typedef {Object} NodeInfo
 * @property {number[]} path   [segment, child …]
 * @property {number} depth    0 = segment.
 * @property {Rat} start       Onset in quarters from the top of the bar.
 * @property {Rat} span        Real length in quarters.
 * @property {Rat} written     Written length in quarters (what the engraver draws).
 * @property {number} n
 * @property {{n: number, m: (number|null), w: Rat} | null} tuplet
 * @property {boolean} leaf
 */

/**
 * Every node of a bar, depth-first. Leaves come out in time order, which is
 * the order of the lane hit strings.
 *
 * @param {Bar} bar
 * @param {RNode[]} [segs]  A lane's effective segments (see laneSegs).
 * @returns {NodeInfo[]}
 */
export function barNodes(bar, segs = bar.segs) {
  /** @type {NodeInfo[]} */
  const out = [];
  let g = 0;
  let onset = rat(0);
  segs.forEach((seg, si) => {
    let units = 0;
    for (let k = g; k < g + seg.w; k++) units += bar.groups[k];
    g += seg.w;
    const span = rat(units * 4, bar.den);
    walk(seg, onset, span, span, [si], 0, out);
    onset = radd(onset, span);
  });
  return out;
}

/**
 * @param {RNode} node @param {Rat} onset @param {Rat} span @param {Rat} wspan
 * @param {number[]} path @param {number} depth @param {NodeInfo[]} out
 */
function walk(node, onset, span, wspan, path, depth, out) {
  const isLeaf = node.n === 1 && !node.kids;
  const t = tupletFor(wspan, node.n, node.m);
  out.push({ path, depth, start: onset, span, written: wspan, n: node.n, tuplet: t, leaf: isLeaf });
  if (isLeaf) return;
  const slot = rdiv(span, rat(node.n));
  const wslot = t ? t.w : rdiv(wspan, rat(node.n));
  if (!node.kids) {
    for (let i = 0; i < node.n; i++) {
      out.push({
        path: [...path, i],
        depth: depth + 1,
        start: radd(onset, rmul(slot, rat(i))),
        span: slot,
        written: wslot,
        n: 1,
        tuplet: null,
        leaf: true,
      });
    }
    return;
  }
  let s = 0;
  node.kids.forEach((kid, ki) => {
    walk(
      kid,
      radd(onset, rmul(slot, rat(s))),
      rmul(slot, rat(kid.w)),
      rmul(wslot, rat(kid.w)),
      [...path, ki],
      depth + 1,
      out
    );
    s += kid.w;
  });
}

/** @param {Bar} bar @param {RNode[]} [segs] @returns {NodeInfo[]} */
export function barLeaves(bar, segs) {
  return barNodes(bar, segs).filter((x) => x.leaf);
}

/**
 * A lane's segments for one bar, with its overrides grafted in.
 *
 * @param {Score} score
 * @param {string} lane
 * @param {number} barIndex
 * @returns {RNode[]}
 */
export function laneSegs(score, lane, barIndex) {
  const bar = score.bars[barIndex];
  const ovs = score.overrides.filter((o) => o.lane === lane && o.bar === barIndex);
  if (!ovs.length) return bar.segs;
  const segs = bar.segs.map(cloneNode);
  for (const o of ovs) graft(segs, o.path, o.node);
  return segs;
}

/** @param {RNode[]} segs @param {number[]} path @param {RNode} node */
function graft(segs, path, node) {
  let list = segs;
  for (let d = 0; d < path.length; d++) {
    const idx = path[d];
    const target = list[idx];
    if (!target) throw new RangeError(`override path ${path.join('.')} does not exist`);
    if (d === path.length - 1) {
      list[idx] = { ...cloneNode(node), w: target.w };
      return;
    }
    if (!target.kids) target.kids = Array.from({ length: target.n }, () => ({ n: 1, w: 1 }));
    list = target.kids;
  }
}

/* ------------------------------------------------------------------ */
/* Text form of nodes and bars                                         */
/* ------------------------------------------------------------------ */

//   node  := INT [ ':' INT ] [ '(' node { ',' node } ')' ] [ '@' INT ]
//   bar   := INT '/' INT [ '~' INT { '-' INT } ] ':' node { ',' node } [ '*' INT ]
//
//   4              four leaves
//   6:4            six leaves, bracket labelled 6:4
//   3:2(2,2,5:4)   three slots: two split in 2, the third holds a 5:4
//   5:4(1,1,1,3:2@2)   five slots; the 3:2 fills slots 4–5
//   7/8~2-2-3:4,4,6    a 7/8 bar in 16ths (grouping omitted when it's the default)
//   4/4:4*4            four bars of 4/4 in 16ths (one segment repeats across the groups)

class Cursor {
  /** @param {string} s */
  constructor(s) {
    this.s = s;
    this.i = 0;
  }
  /** @param {string} ch */
  eat(ch) {
    if (this.s[this.i] === ch) {
      this.i++;
      return true;
    }
    return false;
  }
  int() {
    const start = this.i;
    while (this.i < this.s.length && this.s[this.i] >= '0' && this.s[this.i] <= '9') this.i++;
    if (start === this.i) {
      throw new SyntaxError(`expected a number at ${start} in '${this.s}'`);
    }
    return Number(this.s.slice(start, this.i));
  }
  done() {
    return this.i >= this.s.length;
  }
}

/** @param {Cursor} c @param {number} depth @returns {RNode} */
function parseNodeAt(c, depth) {
  if (depth > MAX_DEPTH) throw new SyntaxError(`nested deeper than ${MAX_DEPTH} levels`);
  const n = c.int();
  if (n < 1 || n > MAX_SLOTS) throw new SyntaxError(`a split of ${n} is out of range`);
  /** @type {RNode} */
  const node = { n, w: 1 };
  if (c.eat(':')) {
    node.m = c.int();
    if (node.m < 1 || node.m > MAX_SLOTS) throw new SyntaxError(`ratio ${n}:${node.m}`);
  }
  if (c.eat('(')) {
    const kids = parseKids(c, depth, n);
    // Canonical form: n(1,1,…) is just n.
    if (!kids.every((k) => k.n === 1 && k.w === 1 && !k.m && !k.kids)) node.kids = kids;
  }
  if (c.eat('@')) {
    node.w = c.int();
    if (node.w < 1) throw new SyntaxError('weight must be at least 1');
  }
  return node;
}

/** After '(': children up to ')', whose weights must fill n slots. @returns {RNode[]} */
function parseKids(c, depth, n) {
  const kids = [];
  do kids.push(parseNodeAt(c, depth + 1));
  while (c.eat(','));
  if (!c.eat(')')) throw new SyntaxError(`expected ')' at ${c.i} in '${c.s}'`);
  const filled = kids.reduce((a, k) => a + k.w, 0);
  if (filled !== n) throw new SyntaxError(`children of ${n}(…) fill ${filled} slots, not ${n}`);
  return kids;
}

/** @param {string} s @returns {RNode} */
export function parseNode(s) {
  const c = new Cursor(s);
  const node = parseNodeAt(c, 0);
  if (!c.done()) throw new SyntaxError(`unexpected '${s[c.i]}' at ${c.i} in '${s}'`);
  return node;
}

/** @param {RNode} node @returns {string} */
export function formatNode(node) {
  let s = String(node.n);
  if (node.m) s += ':' + node.m;
  if (node.kids) s += '(' + node.kids.map(formatNode).join(',') + ')';
  if (node.w !== 1) s += '@' + node.w;
  return s;
}

const VALID_DENS = new Set([1, 2, 4, 8, 16, 32]);

/**
 * @param {string} s
 * @returns {{bar: Bar, count: number}}
 */
export function parseBar(s) {
  const m = /^(\d+)\/(\d+)(?:~(\d+(?:-\d+)*))?:(.+?)(?:\*(\d+))?$/.exec(s);
  if (!m) throw new SyntaxError(`'${s}' is not a bar (expected like 4/4:4)`);
  const num = Number(m[1]);
  const den = Number(m[2]);
  if (num < 1 || num > 64) throw new SyntaxError(`meter ${num}/${den}: numerator out of range`);
  if (!VALID_DENS.has(den)) throw new SyntaxError(`meter ${num}/${den}: denominator out of range`);
  const groups = m[3] ? m[3].split('-').map(Number) : defaultGroups(num, den);
  if (groups.some((g) => g < 1) || groups.reduce((a, b) => a + b, 0) !== num) {
    throw new SyntaxError(`grouping ${groups.join('+')} doesn't add up to ${num}`);
  }
  const c = new Cursor(m[4]);
  let segs = [];
  do segs.push(parseNodeAt(c, 0));
  while (c.eat(','));
  if (!c.done()) throw new SyntaxError(`unexpected '${m[4][c.i]}' in bar '${s}'`);
  if (segs.length === 1 && segs[0].w === 1 && groups.length > 1) {
    segs = groups.map(() => cloneNode(segs[0]));
  }
  const used = segs.reduce((a, sg) => a + sg.w, 0);
  if (used !== groups.length) {
    throw new SyntaxError(`segments cover ${used} beat groups; ${num}/${den} has ${groups.length}`);
  }
  const bar = { num, den, groups, segs };
  const leaves = barLeaves(bar).length;
  if (leaves > MAX_LEAVES_PER_BAR) throw new SyntaxError(`${leaves} slots in one bar is too many`);
  return { bar, count: m[5] ? Math.max(1, Number(m[5])) : 1 };
}

/** @param {Bar} bar @param {number} [count] @returns {string} */
export function formatBar(bar, count = 1) {
  const dflt = defaultGroups(bar.num, bar.den);
  const sameGroups = dflt.length === bar.groups.length && dflt.every((g, i) => g === bar.groups[i]);
  const segs = bar.segs.map(formatNode);
  const repeated =
    segs.length > 1 &&
    segs.length === bar.groups.length &&
    segs.every((x) => x === segs[0]) &&
    bar.segs.every((sg) => sg.w === 1);
  return (
    `${bar.num}/${bar.den}` +
    (sameGroups ? '' : '~' + bar.groups.join('-')) +
    ':' +
    (repeated ? segs[0] : segs.join(',')) +
    (count > 1 ? '*' + count : '')
  );
}

/* ------------------------------------------------------------------ */
/* Lane hit strings                                                    */
/* ------------------------------------------------------------------ */

// One char per leaf, '-' = rest. Runs of 3+ rests compress to '-N'. A bar that
// is all rests is empty; a bar identical to the previous one is '.'. Bars are
// separated by ';' and trailing empty bars are dropped.

/** @param {string} s */
function rle(s) {
  return s.replace(/-{3,}/g, (run) => '-' + run.length);
}

/** @param {string} s */
function unrle(s) {
  return s.replace(/-(\d+)/g, (_, k) => '-'.repeat(Math.min(Number(k), MAX_LEAVES_PER_BAR)));
}

/** @param {string[]} strs */
function encodeLane(strs) {
  const parts = strs.map((s, i) => {
    if (!/[^-]/.test(s)) return '';
    if (i > 0 && s === strs[i - 1]) return '.';
    return rle(s);
  });
  while (parts.length && parts[parts.length - 1] === '') parts.pop();
  return parts.join(';');
}

/* ------------------------------------------------------------------ */
/* URL codec                                                           */
/* ------------------------------------------------------------------ */

/**
 * Encode a score as a URL fragment (without the leading '#').
 *
 * @param {Score} score
 * @returns {string}
 */
export function encode(score) {
  const p = ['v=2'];
  if (score.title) p.push('t=' + encodeURIComponent(score.title));
  if (score.author) p.push('a=' + encodeURIComponent(score.author));
  if (score.comments) p.push('c=' + encodeURIComponent(score.comments));
  p.push('q=' + score.tempo);
  if (score.swing) p.push('sw=' + score.swing);

  const bars = [];
  for (let i = 0; i < score.bars.length;) {
    const s = formatBar(score.bars[i]);
    let k = 1;
    while (i + k < score.bars.length && formatBar(score.bars[i + k]) === s) k++;
    bars.push(formatBar(score.bars[i], k));
    i += k;
  }
  p.push('b=' + bars.join(';'));

  if (score.tempoChanges.length) {
    p.push(
      'tc=' + score.tempoChanges.map((c) => `${c.bar}:${rstr(c.unit)}~${rstr(c.old)}`).join(';')
    );
  }
  if (score.overrides.length) {
    p.push(
      'ov=' +
        score.overrides
          .map(
            (o) => `${o.lane}@${[o.bar, ...o.path].join('.')}!${formatNode({ ...o.node, w: 1 })}`
          )
          .join(';')
    );
  }
  const lanes = [
    ...LANE_ORDER.filter((id) => score.hits[id]),
    ...Object.keys(score.hits)
      .filter((id) => !(id in LANES))
      .sort(),
  ];
  for (const id of lanes) {
    const v = encodeLane(score.hits[id]);
    if (v) p.push(id + '=' + v);
  }
  return p.join('&');
}

/** @typedef {{level: ('error'|'warn'), msg: string}} Issue */

/** @param {string} s */
function safeDecode(s) {
  try {
    return decodeURIComponent(s);
  } catch {
    return s;
  }
}

/** @param {string} body @returns {Map<string, string>} */
function parseParams(body) {
  const map = new Map();
  for (const part of body.split('&')) {
    if (!part) continue;
    const eq = part.indexOf('=');
    const key = eq < 0 ? part : part.slice(0, eq);
    map.set(key, eq < 0 ? '' : part.slice(eq + 1));
  }
  return map;
}

const LEGACY_KEYS = new Set(['timesig', 'div', 'h', 's', 'k', 'measures']);

/**
 * The part of a URL that holds the groove: a v2 fragment wins, then the query
 * string (legacy links, or v2 sent as a query), then the input itself.
 *
 * @param {string} s
 */
function grooveBody(s) {
  const hashAt = s.indexOf('#');
  const qAt = s.indexOf('?');
  const hash = hashAt >= 0 ? s.slice(hashAt + 1) : '';
  if (hashAt >= 0 && /(^|&)v=2(&|$)/.test(hash)) return hash;
  if (qAt < 0) return hashAt >= 0 ? hash : s;
  return hashAt > qAt ? s.slice(qAt + 1, hashAt) : s.slice(qAt + 1);
}

/**
 * Run `fn` on each ';'-separated item of a field; a throw goes to `onError`
 * and the next item is still read.
 *
 * @param {string|undefined} raw
 * @param {(item: string) => void} fn
 * @param {(e: Error) => void} onError
 */
function eachItem(raw, fn, onError) {
  for (const item of safeDecode(raw ?? '').split(';')) {
    if (!item) continue;
    try {
      fn(item);
    } catch (e) {
      onError(e);
    }
  }
}

/** @param {Score} score @param {Map<string, string>} params @param {Issue[]} issues */
function decodeMeta(score, params, issues) {
  score.title = safeDecode(params.get('t') ?? '');
  score.author = safeDecode(params.get('a') ?? '');
  score.comments = safeDecode(params.get('c') ?? '');
  const q = Number(safeDecode(params.get('q') ?? '80'));
  if (Number.isFinite(q) && q >= 10 && q <= 999) score.tempo = q;
  else issues.push({ level: 'warn', msg: `tempo '${params.get('q')}' out of range; using 80` });
  const sw = Number(params.get('sw') ?? 0);
  score.swing = Number.isFinite(sw) && sw >= 0 && sw <= 100 ? sw : 0;
}

/** @param {Score} score @param {string|undefined} raw @param {Issue[]} issues */
function decodeBars(score, raw, issues) {
  score.bars = [];
  eachItem(
    raw ?? '4/4:4',
    (spec) => {
      const { bar, count } = parseBar(spec);
      for (let k = 0; k < count && score.bars.length < MAX_BARS; k++) {
        score.bars.push({ ...bar, groups: [...bar.groups], segs: bar.segs.map(cloneNode) });
      }
    },
    (e) => {
      issues.push({ level: 'error', msg: `${e.message}; replaced with 4/4 in 16ths` });
      score.bars.push(defaultBar(4, 4));
    }
  );
  if (!score.bars.length) score.bars.push(defaultBar(4, 4));
}

/** @param {Score} score @param {string|undefined} raw @param {Issue[]} issues */
function decodeTempoChanges(score, raw, issues) {
  eachItem(
    raw,
    (tc) => {
      const m = /^(\d+):([\d/]+)~([\d/]+)$/.exec(tc);
      if (!m) throw new SyntaxError(`'${tc}' is not bar:unit~old`);
      score.tempoChanges.push({ bar: Number(m[1]), unit: parseRat(m[2]), old: parseRat(m[3]) });
    },
    (e) => issues.push({ level: 'warn', msg: `tempo change ignored: ${e.message}` })
  );
}

/** @param {Score} score @param {string|undefined} raw @param {Issue[]} issues */
function decodeOverrides(score, raw, issues) {
  eachItem(
    raw,
    (ov) => {
      const m = /^([a-z][a-z0-9]*)@(\d+(?:\.\d+)+)!(.+)$/.exec(ov);
      if (!m) throw new SyntaxError(`'${ov}' is not lane@bar.path!node`);
      const [bar, ...path] = m[2].split('.').map(Number);
      if (bar >= score.bars.length) throw new RangeError(`bar ${bar} doesn't exist`);
      const o = { lane: m[1], bar, path, node: parseNode(m[3]) };
      laneSegs({ ...score, overrides: [...score.overrides, o] }, o.lane, o.bar); // throws on a bad path
      score.overrides.push(o);
    },
    (e) => issues.push({ level: 'warn', msg: `override ignored: ${e.message}` })
  );
}

/**
 * One bar of one lane: expand '-N' runs and '.', turn letters the lane doesn't
 * know into rests, and fit the string to the lane's slot count.
 *
 * @param {{key: string, b: number, want: number, allowed: (string|null)}} at
 * @param {string} part
 * @param {string} prev  The previous bar's decoded string, for '.'.
 * @param {Issue[]} issues
 */
function decodeLaneBar(at, part, prev, issues) {
  const { key, b, want, allowed } = at;
  let str = part === '.' ? prev : unrle(part);
  const bad = allowed ? [...str].filter((ch) => !allowed.includes(ch)) : [];
  if (bad.length) {
    issues.push({ level: 'warn', msg: `${key} bar ${b}: '${bad.join('')}' not valid; made rests` });
    str = [...str].map((ch) => (allowed.includes(ch) ? ch : '-')).join('');
  }
  if (str.length !== want && part !== '') {
    const fix = str.length > want ? 'cut' : 'padded';
    issues.push({
      level: 'warn',
      msg: `${key} bar ${b}: ${str.length} hits for ${want} slots; ${fix}`,
    });
  }
  return (str + '-'.repeat(want)).slice(0, want);
}

/** @param {Score} score @param {string} key @param {string} raw @param {Issue[]} issues */
function decodeLane(score, key, raw, issues) {
  const known = LANES[key];
  if (!known) issues.push({ level: 'warn', msg: `custom lane '${key}' kept as-is` });
  const allowed = known ? known.chars + '-' : null;
  const parts = safeDecode(raw).split(';');
  if (parts.length > score.bars.length) {
    issues.push({
      level: 'warn',
      msg: `${key}: hits for ${parts.length} bars, groove has ${score.bars.length}`,
    });
  }
  /** @type {string[]} */
  const strs = [];
  for (let b = 0; b < score.bars.length; b++) {
    const want = barLeaves(score.bars[b], laneSegs(score, key, b)).length;
    strs.push(decodeLaneBar({ key, b, want, allowed }, parts[b] ?? '', strs[b - 1] ?? '', issues));
  }
  score.hits[key] = strs;
}

/**
 * Decode a v2 fragment, a full URL, or a legacy Groove Scribe query string.
 * Never throws for bad input: problems come back as issues, and whatever could
 * be read is kept.
 *
 * @param {string} input
 * @returns {{score: Score, issues: Issue[], legacy: boolean}}
 */
export function decode(input) {
  /** @type {Issue[]} */
  const issues = [];
  const params = parseParams(grooveBody(String(input).trim()));
  if (params.get('v') !== '2') {
    const keys = [...params.keys()].map((k) => k.toLowerCase());
    if (keys.some((k) => LEGACY_KEYS.has(k))) {
      return { score: fromLegacy(params, issues), issues, legacy: true };
    }
    issues.push({ level: 'warn', msg: 'no v=2 and no Groove Scribe fields; starting empty' });
    return { score: emptyScore(), issues, legacy: false };
  }

  const score = emptyScore();
  decodeMeta(score, params, issues);
  decodeBars(score, params.get('b'), issues);
  decodeTempoChanges(score, params.get('tc'), issues);
  // Overrides before hits: they change how many leaves a lane has.
  decodeOverrides(score, params.get('ov'), issues);
  for (const [key, raw] of params) {
    if (META_KEYS.has(key)) continue;
    if (/^[a-z][a-z0-9]{1,5}$/.test(key)) decodeLane(score, key, raw, issues);
    else issues.push({ level: 'warn', msg: `unknown field '${key}' ignored` });
  }
  return { score, issues, legacy: false };
}

/* ------------------------------------------------------------------ */
/* Legacy Groove Scribe import                                         */
/* ------------------------------------------------------------------ */

/** Groove Scribe tab letter → [lane, char] pairs, per legacy field. */
const LEGACY_MAP = {
  H: {
    x: [['hh', 'x']],
    X: [['hh', 'X']],
    o: [['hh', 'o']],
    '+': [['hh', '+']],
    c: [['cr', 'x']],
    r: [['rd', 'x']],
    R: [['rd', 'x']],
    b: [['rd', 'b']],
    B: [['rd', 'b']],
    s: [['sk', 'x']],
    m: [['cb', 'x']],
    n: [['mt', 'n']],
    N: [['mt', 'N']],
  },
  S: {
    o: [['sn', 'o']],
    O: [['sn', 'O']],
    g: [['sn', 'g']],
    x: [['sn', 'x']],
    f: [['sn', 'f']],
    d: [['sn', 'd']],
    b: [['sn', 'b']],
    B: [['sn', 'b']],
  },
  K: {
    o: [['kk', 'o']],
    x: [['hf', 'x']],
    X: [
      ['kk', 'o'],
      ['hf', 'x'],
    ],
  },
  T1: { o: [['t1', 'o']], x: [['t1', 'o']] },
  T2: { o: [['t2', 'o']] },
  T3: { o: [['t3', 'o']] },
  T4: { o: [['t4', 'o']], x: [['t4', 'o']] },
  Stickings: {
    R: [['st', 'R']],
    r: [['st', 'R']],
    L: [['st', 'L']],
    l: [['st', 'L']],
    B: [['st', 'B']],
    b: [['st', 'B']],
    c: [['st', 'c']],
  },
};

/** Mirrors noteArraysFromURLData's tolerant re-scaling of over/under-long tab lines. */
function legacyCells(raw, size) {
  const notes = safeDecode(raw).replace(/[:!)([\]|]/g, '');
  let take = 1;
  let spread = 1;
  if (notes.length > size && notes.length / size >= 2) take = Math.ceil(notes.length / size);
  else if (notes.length < size && size / notes.length >= 2) spread = Math.ceil(size / notes.length);
  const cells = Array(size).fill('-');
  for (let j = 0, k = 0; j < notes.length && k < size; j += take, k += spread) cells[k] = notes[j];
  return cells;
}

/**
 * Convert a Groove Scribe query (TimeSig, Div, Measures, H, S, K, T1–T4,
 * Stickings, Title, Author, Comments, Tempo, Swing) to a v2 score. Field names
 * are case-insensitive, as in getQueryVariableFromString.
 *
 * @param {Map<string, string>} params
 * @param {Issue[]} issues
 * @returns {Score}
 */
export function fromLegacy(params, issues = []) {
  const get = (name, dflt) => {
    for (const [k, v] of params) if (k.toLowerCase() === name.toLowerCase()) return v;
    return dflt;
  };
  const score = emptyScore();
  const [tsTop, tsBottom] = String(get('TimeSig', '4/4')).split('/').map(Number);
  const num = tsTop >= 1 && tsTop <= 32 ? tsTop : 4;
  const den = [2, 4, 8, 16].includes(tsBottom) ? tsBottom : 4;
  const div = Number.parseInt(get('Div', '16'), 10) || 16;
  let measures = Number.parseInt(get('Measures', '1'), 10);
  if (Number.isNaN(measures) || measures < 1) measures = 1;
  if (measures > 10) measures = 10;

  let perBar = (div / den) * num;
  if (!Number.isInteger(perBar)) {
    issues.push({ level: 'warn', msg: `Div=${div} doesn't divide ${num}/${den} evenly; rounded` });
    perBar = Math.max(1, Math.round(perBar));
  }
  const groups = defaultGroups(num, den);
  const per = groups.map((g) => (perBar * g) / num);
  const bar = per.every(Number.isInteger)
    ? { num, den, groups, segs: per.map((n) => ({ n, w: 1 })) }
    : { num, den, groups, segs: [{ n: perBar, w: groups.length }] };
  score.bars = Array.from({ length: measures }, () => ({
    ...bar,
    groups: [...groups],
    segs: bar.segs.map(cloneNode),
  }));

  const text = (k) => {
    const v = get(k, '');
    return v ? safeDecode(v).replace(/\+/g, ' ') : '';
  };
  score.title = text('Title');
  score.author = text('Author');
  score.comments = text('Comments');
  const tempo = Number.parseInt(get('Tempo', '80'), 10);
  score.tempo = tempo >= 20 && tempo <= 400 ? tempo : 80;
  const swing = Number.parseInt(get('Swing', '0'), 10);
  score.swing = swing >= 0 && swing <= 100 ? swing : 0;

  const rows = {};
  for (const field of Object.keys(LEGACY_MAP)) {
    const raw = get(field, '');
    if (!raw) continue;
    const cells = legacyCells(raw, perBar * measures);
    cells.forEach((ch, i) => {
      if (ch === '-') return;
      const targets = LEGACY_MAP[field][ch];
      if (!targets) {
        issues.push({ level: 'warn', msg: `${field}: tab letter '${ch}' unknown; dropped` });
        return;
      }
      for (const [lane, art] of targets) {
        rows[lane] ??= Array(perBar * measures).fill('-');
        rows[lane][i] = art;
      }
    });
  }
  for (const [lane, cells] of Object.entries(rows)) {
    score.hits[lane] = Array.from({ length: measures }, (_, b) =>
      cells.slice(b * perBar, (b + 1) * perBar).join('')
    );
  }
  return score;
}

/* ------------------------------------------------------------------ */
/* Playback and MIDI resolution                                        */
/* ------------------------------------------------------------------ */

/**
 * @typedef {Object} PlayEvent
 * @property {number} t     Seconds from the start of the groove.
 * @property {string} lane
 * @property {string} art   Articulation char.
 * @property {number} bar
 * @property {number} leaf
 */

/**
 * Flatten a score into time-sorted events for a Web Audio scheduler. Exact
 * rationals until the last step, so long loops of nested tuplets don't drift.
 *
 * @param {Score} score
 * @returns {{events: PlayEvent[], duration: number}}
 */
export function compileEvents(score) {
  /** @type {PlayEvent[]} */
  const events = [];
  const changes = new Map(score.tempoChanges.map((c) => [c.bar, c]));
  let tempo = score.tempo;
  let t0 = 0;
  const lanes = Object.keys(score.hits);
  score.bars.forEach((bar, b) => {
    const tc = changes.get(b);
    if (tc) tempo = (tempo * rnum(tc.unit)) / rnum(tc.old);
    const spq = 60 / tempo;
    for (const lane of lanes) {
      const s = score.hits[lane][b];
      if (!s || !/[^-]/.test(s)) continue;
      const leaves = barLeaves(bar, laneSegs(score, lane, b));
      for (let i = 0; i < s.length && i < leaves.length; i++) {
        if (s[i] !== '-') {
          events.push({ t: t0 + rnum(leaves[i].start) * spq, lane, art: s[i], bar: b, leaf: i });
        }
      }
    }
    t0 += rnum(barLength(bar)) * spq;
  });
  events.sort((x, y) => x.t - y.t || LANE_ORDER.indexOf(x.lane) - LANE_ORDER.indexOf(y.lane));
  return { events, duration: t0 };
}

/**
 * The MIDI ticks-per-quarter that puts every onset on an integer tick.
 * Groove Scribe writes 128 PPQ today, which can't place a 16th-note triplet
 * (it uses 10.666 ticks). `exact` is the minimum; `ppq` is the smallest
 * multiple of it that is at least 480, and `fits` says whether it is within
 * the 15-bit limit of a MIDI header.
 *
 * @param {Score} score
 * @returns {{exact: number, ppq: number, fits: boolean}}
 */
export function midiResolution(score) {
  let l = 1;
  score.bars.forEach((bar, b) => {
    const lanes = Object.keys(score.hits);
    const segSets = lanes.length ? lanes.map((lane) => laneSegs(score, lane, b)) : [bar.segs];
    for (const segs of segSets) {
      for (const leaf of barLeaves(bar, segs)) l = lcm(l, leaf.start[1]);
    }
    l = lcm(l, barLength(bar)[1]);
  });
  const ppq = l * Math.ceil(480 / l);
  return { exact: l, ppq, fits: ppq <= 32767 };
}
