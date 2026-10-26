export type Rat = { num: number; den: number };
/** `start` is in quarters from the top of the bar. */
export type Leaf = { segment: number; start: Rat; span: Rat };
export type BarView = { meter: string; length: Rat; leaves: Leaf[] };
export type GrooveView = { bars: BarView[] };
export type Issue = { level: 'error' | 'warn'; msg: string };
export type Core = {
  openLink(input: string): { issues: Issue[] };
  view(link: string): GrooveView;
};
