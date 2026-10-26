import type { BarView, GrooveView, Issue, Rat } from './core/types';

const quarters = (r: Rat) => r.num / r.den;

export function Groove({ view, issues }: { view: GrooveView; issues: Issue[] }) {
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
      <div className="groove">
        {view.bars.map((bar, i) => (
          <Bar key={i} bar={bar} />
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
            className={i === 0 || bar.leaves[i - 1].segment !== leaf.segment ? 'slot segment-start' : 'slot'}
            data-slot
            style={{ flexGrow: quarters(leaf.span) }}
          />
        ))}
      </div>
    </div>
  );
}
