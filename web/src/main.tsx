import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import { App } from './App';
import { core } from './core/core';
import init from './core/pkg/rhythm_tree_wasm.js';
import './styles.css';

void init().then(() => {
  createRoot(document.getElementById('root')!).render(
    <StrictMode>
      <App core={core} href={location.href} />
    </StrictMode>,
  );
});
