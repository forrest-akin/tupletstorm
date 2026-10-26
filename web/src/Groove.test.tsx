// @vitest-environment jsdom
import { afterEach, expect, test } from 'vitest';
import { cleanup, render, screen } from '@testing-library/react';
import { Groove } from './Groove';
import type { BarView, Leaf } from './core/types';

afterEach(cleanup);

const quarter = { num: 1, den: 4 };

function bar(meter: string, length: BarView['length'], segments: number[]): BarView {
  const leaves: Leaf[] = segments.map((segment, i) => ({ segment, start: { num: i, den: 4 }, span: quarter }));
  return { meter, length, leaves };
}

const fourFour = bar('4/4', { num: 4, den: 1 }, [0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3]);
const sevenEight = bar('7/8', { num: 7, den: 2 }, [0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2]);

test('draws a row of slots per bar, sized by length', () => {
  const { container } = render(<Groove view={{ bars: [fourFour, sevenEight] }} issues={[]} />);

  expect(container.querySelectorAll('[data-slot]')).toHaveLength(30);
  const labels = [screen.getByText('4/4'), screen.getByText('7/8')];
  const bars = labels.map((label) => label.closest('[data-bar]') as HTMLElement);
  expect(bars.map((b) => b.style.flexGrow)).toEqual(['4', '3.5']);
  const slots = [...bars[0].querySelectorAll<HTMLElement>('[data-slot]')];
  expect(slots[0].style.flexGrow).toBe('0.25');
  const starts = slots.flatMap((slot, i) => (slot.classList.contains('segment-start') ? [i] : []));
  expect(starts).toEqual([0, 4, 8, 12]);
});

test('lists the issues from opening the link', () => {
  render(<Groove view={{ bars: [] }} issues={[{ level: 'error', msg: 'x; replaced with 4/4 in 16ths' }]} />);

  expect(screen.getByText('x; replaced with 4/4 in 16ths')).toBeTruthy();
});
