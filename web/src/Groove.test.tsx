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
  const { container } = render(<Groove view={{ bars: [fourFour, sevenEight], lanes: [] }} issues={[]} />);

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
  render(<Groove view={{ bars: [], lanes: [] }} issues={[{ level: 'error', msg: 'x; replaced with 4/4 in 16ths' }]} />);

  expect(screen.getByText('x; replaced with 4/4 in 16ths').className).toBe('error');
});

test('draws a row per lane with its letters', () => {
  const lanes = [
    { id: 'hh', name: 'Hi-hat', letters: '-xXo+', bars: [{ hits: 'x-x-x-x-x-x-x-x-' }] },
    { id: 'sn', name: 'Snare', letters: '-oOgxfdb', bars: [{ hits: '----o-------o---' }] },
  ];
  render(<Groove view={{ bars: [fourFour], lanes }} issues={[]} />);

  const row = (name: string) => screen.getByText(name).closest('[data-lane]') as HTMLElement;
  expect(row('Hi-hat').querySelectorAll('[data-cell]')).toHaveLength(16);
  expect(row('Snare').querySelectorAll('[data-cell]')).toHaveLength(16);
  expect(row('Hi-hat').querySelector('[data-cell]')?.textContent).toBe('x');
  expect(screen.getByLabelText('Hi-hat, bar 1, slot 1: x')).toBeTruthy();
  expect(screen.getByLabelText('Snare, bar 1, slot 5: o')).toBeTruthy();
  expect(screen.getByLabelText('Snare, bar 1, slot 1: rest').textContent).toBe('');
  const cellsRow = row('Hi-hat').querySelector<HTMLElement>('.cells')!;
  expect(cellsRow.style.flexGrow).toBe('4');
  const cells = [...cellsRow.querySelectorAll<HTMLElement>('[data-cell]')];
  expect(cells[0].style.flexGrow).toBe('0.25');
  const starts = cells.flatMap((cell, i) => (cell.classList.contains('segment-start') ? [i] : []));
  expect(starts).toEqual([0, 4, 8, 12]);
});

test('gives each code point of a lane its own cell', () => {
  const lanes = [{ id: 'zz9', name: 'zz9', letters: '', bars: [{ hits: '😀x--------------' }] }];
  render(<Groove view={{ bars: [fourFour], lanes }} issues={[]} />);

  expect(screen.getByLabelText('zz9, bar 1, slot 1: 😀')).toBeTruthy();
  expect(screen.getByLabelText('zz9, bar 1, slot 2: x')).toBeTruthy();
  expect(screen.getByLabelText('zz9, bar 1, slot 3: rest')).toBeTruthy();
});
