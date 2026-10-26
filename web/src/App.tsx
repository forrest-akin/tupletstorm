import { useState } from 'react';
import { Groove } from './Groove';
import type { Core } from './core/types';

export function App({ core, href }: { core: Core; href: string }) {
  const [{ link, lanes, issues }] = useState(() => core.openLink(href));
  return <Groove view={core.view(link, lanes)} issues={issues} />;
}
