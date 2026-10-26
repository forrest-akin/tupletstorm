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
    openLink: vi.fn(() => ({ link: 'v=2&q=80&b=4/4:4', lanes: [], issues: [{ level: 'error' as const, msg: 'x; replaced with 4/4 in 16ths' }] })),
    view: vi.fn(() => ({ title: '', tempo: 80, bars: [fourFour], lanes: [] })),
  };

  const { container } = render(<App core={core} href={href} />);

  expect(core.openLink).toHaveBeenCalledWith(href);
  expect(screen.getByText('x; replaced with 4/4 in 16ths')).toBeTruthy();
  expect(container.querySelectorAll('[data-slot]')).toHaveLength(16);
});

test('passes the opened lanes to view', () => {
  const href = 'https://x.test/#v=2&b=4/4:4';
  const core: Core = {
    openLink: vi.fn(() => ({ link: 'v=2&q=80&b=4/4:4', lanes: ['hh', 'sn'], issues: [] })),
    view: vi.fn(() => ({ title: '', tempo: 80, bars: [fourFour], lanes: [] })),
  };

  render(<App core={core} href={href} />);

  expect(core.view).toHaveBeenCalledWith('v=2&q=80&b=4/4:4', ['hh', 'sn']);
});

test('views the canonical link', () => {
  const core: Core = {
    openLink: vi.fn(() => ({ link: 'v=2&q=80&b=4/4:4', lanes: [], issues: [] })),
    view: vi.fn(() => ({ title: '', tempo: 80, bars: [fourFour], lanes: [] })),
  };

  render(<App core={core} href="https://x.test/#v=2&b=4/4:4" />);

  expect(core.view).toHaveBeenCalledWith('v=2&q=80&b=4/4:4', []);
});
