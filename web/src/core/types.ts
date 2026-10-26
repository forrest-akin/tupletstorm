export type Rat = { num: number; den: number };
/** `start` is in quarters from the top of the bar. */
export type Leaf = { segment: number; start: Rat; span: Rat };
export type BarView = { meter: string; grouping: string | null; length: Rat; leaves: Leaf[] };
export type LaneView = { id: string; name: string; letters: string; bars: { hits: string }[] };
export type GrooveView = { title: string; tempo: number; bars: BarView[]; lanes: LaneView[] };
export type Issue = { level: 'error' | 'warn'; msg: string };
export type Core = {
  openLink(input: string): { link: string; lanes: string[]; issues: Issue[] };
  view(link: string, lanes: string[]): GrooveView;
  setHit(link: string, lane: string, bar: number, leaf: number, letter: string): string;
};
