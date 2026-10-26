// @vitest-environment jsdom
import { afterEach, expect, test, vi } from 'vitest';
import { cleanup, render, screen } from '@testing-library/react';
import { App } from './App';
import type { BarView, Core } from './core/types';

afterEach(cleanup);

const quarter = { num: 1, den: 4 };

const fourFour: BarView = {
  meter: '4/4',
  length: { num: 4, den: 1 },
  leaves: Array.from({ length: 16 }, (_, i) => ({
    segment: Math.floor(i / 4),
    start: { num: i, den: 4 },
    span: quarter,
  })),
};

test("opens the page's link and draws it", () => {
  const href = 'https://x.test/#v=2&b=4/4:4';
  const core: Core = {
    openLink: vi.fn(() => ({ issues: [{ level: 'error' as const, msg: 'x; replaced with 4/4 in 16ths' }] })),
    view: vi.fn(() => ({ bars: [fourFour] })),
  };

  const { container } = render(<App core={core} href={href} />);

  expect(core.openLink).toHaveBeenCalledWith(href);
  expect(core.view).toHaveBeenCalledWith(href);
  expect(screen.getByText('x; replaced with 4/4 in 16ths')).toBeTruthy();
  expect(container.querySelectorAll('[data-slot]')).toHaveLength(16);
});
