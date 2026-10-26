export type Rat = { num: number; den: number };
/** `start` is in quarters from the top of the bar. */
export type Leaf = { segment: number; start: Rat; span: Rat };
export type BarView = { meter: string; length: Rat; leaves: Leaf[] };
export type LaneView = { id: string; name: string; letters: string; bars: { hits: string }[] };
export type GrooveView = { bars: BarView[]; lanes: LaneView[] };
export type Issue = { level: 'error' | 'warn'; msg: string };
export type Core = {
  openLink(input: string): { lanes: string[]; issues: Issue[] };
  view(link: string, lanes: string[]): GrooveView;
};
