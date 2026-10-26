import type { BarView, GrooveView, Issue, LaneView, Rat } from './core/types';

const quarters = (r: Rat) => r.num / r.den;

export type OnCell = (lane: string, bar: number, leaf: number, hit: string, letters: string) => void;

export function Groove({ view, issues, onCell }: { view: GrooveView; issues: Issue[]; onCell: OnCell }) {
  return (
    <main>
      {issues.length > 0 && (
        <ul className="issues">
          {issues.map((issue, i) => (
            <li key={i} className={issue.level}>
              {issue.msg}
            </li>
          ))}
        </ul>
      )}
      <header className="heading">
        {view.title && <h1>{view.title}</h1>}
        <div className="tempo">♩ = {view.tempo}</div>
      </header>
      <div className="groove">
        <div className="row">
          <div className="lane-name" />
          {view.bars.map((bar, i) => (
            <Bar key={i} bar={bar} />
          ))}
        </div>
        {view.lanes.map((lane) => (
          <Lane key={lane.id} lane={lane} bars={view.bars} onCell={onCell} />
        ))}
      </div>
    </main>
  );
}

function Bar({ bar }: { bar: BarView }) {
  return (
    <div className="bar" data-bar style={{ flexGrow: quarters(bar.length) }}>
      <div className="meter">{bar.meter}</div>
      <div className="slots">
        {bar.leaves.map((leaf, i) => (
          <div
            key={i}
            className={segmentStart(bar, i) ? 'slot segment-start' : 'slot'}
            data-slot
            style={{ flexGrow: quarters(leaf.span) }}
          />
        ))}
      </div>
    </div>
  );
}

function Lane({ lane, bars, onCell }: { lane: LaneView; bars: BarView[]; onCell: OnCell }) {
  return (
    <div className="row" data-lane>
      <div className="lane-name">{lane.name}</div>
      {bars.map((bar, b) => {
        const hits = Array.from(lane.bars[b].hits);
        return (
          <div key={b} className="cells" style={{ flexGrow: quarters(bar.length) }}>
            {bar.leaves.map((leaf, i) => {
              const letter = hits[i];
              return (
                <button
                  key={i}
                  type="button"
                  className={segmentStart(bar, i) ? 'cell segment-start' : 'cell'}
                  data-cell
                  aria-label={`${lane.name}, bar ${b + 1}, slot ${i + 1}: ${letter === '-' ? 'rest' : letter}`}
                  style={{ flexGrow: quarters(leaf.span) }}
                  onClick={() => onCell(lane.id, b, i, letter, lane.letters)}
                >
                  {letter === '-' ? '' : letter}
                </button>
              );
            })}
          </div>
        );
      })}
    </div>
  );
}

const segmentStart = (bar: BarView, i: number) => i === 0 || bar.leaves[i - 1].segment !== bar.leaves[i].segment;
