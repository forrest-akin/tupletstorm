import { useState } from 'react';
import { Groove } from './Groove';
import { nextLetter } from './cycle';
import type { Core } from './core/types';

export function App({ core, href, writeLink }: { core: Core; href: string; writeLink: (link: string) => void }) {
  const [{ link: opened, lanes, issues }] = useState(() => core.openLink(href));
  const [link, setLink] = useState(opened);
  const onCell = (lane: string, bar: number, leaf: number, hit: string, letters: string) => {
    const next = core.setHit(link, lane, bar, leaf, nextLetter(letters, hit));
    setLink(next);
    writeLink(next);
  };
  return <Groove view={core.view(link, lanes)} issues={issues} onCell={onCell} />;
}
