import { openLink, setHit, view } from './pkg/rhythm_tree_wasm.js';
import type { Core } from './types';

export const core: Core = { openLink, view, setHit };
