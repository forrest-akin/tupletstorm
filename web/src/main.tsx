import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import { App } from './App';
import { core } from './core/core';
import { linkWriter } from './link';
import init from './core/pkg/rhythm_tree_wasm.js';
import './styles.css';

const writeLink = linkWriter((link) => history.replaceState(null, '', '#' + link), 250).schedule;

void init().then(() => {
  createRoot(document.getElementById('root')!).render(
    <StrictMode>
      <App core={core} href={location.href} writeLink={writeLink} />
    </StrictMode>,
  );
});
