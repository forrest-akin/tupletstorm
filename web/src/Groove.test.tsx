// @vitest-environment jsdom
import { afterEach, expect, test, vi } from 'vitest';
import { cleanup, fireEvent, render, screen } from '@testing-library/react';
import { Groove } from './Groove';
import type { BarView, Leaf } from './core/types';

afterEach(cleanup);

const quarter = { num: 1, den: 4 };

function bar(meter: string, length: BarView['length'], segments: number[], grouping: string | null = null): BarView {
  const leaves: Leaf[] = segments.map((segment, i) => ({ segment, start: { num: i, den: 4 }, span: quarter }));
  return { meter, grouping, length, leaves };
}

const fourFour = bar('4/4', { num: 4, den: 1 }, [0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3]);
const sevenEight = bar('7/8', { num: 7, den: 2 }, [0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2]);

test('draws a row of slots per bar, sized by length', () => {
  const { container } = render(<Groove view={{ title: '', tempo: 80, bars: [fourFour, sevenEight], lanes: [] }} issues={[]} onCell={vi.fn()} />);

  expect(container.querySelectorAll('[data-slot]')).toHaveLength(30);
  expect(screen.queryByRole('heading')).toBeNull();
  const labels = [screen.getByText('4/4'), screen.getByText('7/8')];
  const bars = labels.map((label) => label.closest('[data-bar]') as HTMLElement);
  expect(bars.map((b) => b.style.flexGrow)).toEqual(['4', '3.5']);
  const slots = [...bars[0].querySelectorAll<HTMLElement>('[data-slot]')];
  expect(slots[0].style.flexGrow).toBe('0.25');
  const starts = slots.flatMap((slot, i) => (slot.classList.contains('segment-start') ? [i] : []));
  expect(starts).toEqual([0, 4, 8, 12]);
});

test("shows a bar's grouping when it isn't the default", () => {
  const fiveEight = bar('5/8', { num: 5, den: 2 }, [0, 0, 0, 1, 1], '3+2');
  render(<Groove view={{ title: '', tempo: 80, bars: [fiveEight, fourFour], lanes: [] }} issues={[]} onCell={vi.fn()} />);

  expect(screen.getByText('5/8 3+2')).toBeTruthy();
  expect(screen.getByText('4/4')).toBeTruthy();
});

test('lists the issues from opening the link', () => {
  render(<Groove view={{ title: '', tempo: 80, bars: [], lanes: [] }} issues={[{ level: 'error', msg: 'x; replaced with 4/4 in 16ths' }]} onCell={vi.fn()} />);

  expect(screen.getByText('x; replaced with 4/4 in 16ths').className).toBe('error');
});

test('draws a row per lane with its letters', () => {
  const lanes = [
    { id: 'hh', name: 'Hi-hat', letters: '-xXo+', bars: [{ hits: 'x-x-x-x-x-x-x-x-' }] },
    { id: 'sn', name: 'Snare', letters: '-oOgxfdb', bars: [{ hits: '----o-------o---' }] },
  ];
  render(<Groove view={{ title: '', tempo: 80, bars: [fourFour], lanes }} issues={[]} onCell={vi.fn()} />);

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
  render(<Groove view={{ title: '', tempo: 80, bars: [fourFour], lanes }} issues={[]} onCell={vi.fn()} />);

  expect(screen.getByLabelText('zz9, bar 1, slot 1: 😀')).toBeTruthy();
  expect(screen.getByLabelText('zz9, bar 1, slot 2: x')).toBeTruthy();
  expect(screen.getByLabelText('zz9, bar 1, slot 3: rest')).toBeTruthy();
});

test('shows the title and tempo', () => {
  render(<Groove view={{ title: 'Five over four', tempo: 96, bars: [fourFour], lanes: [] }} issues={[]} onCell={vi.fn()} />);

  expect(screen.getByText('Five over four')).toBeTruthy();
  expect(screen.getByText('♩ = 96')).toBeTruthy();
});

test('reports a clicked cell', () => {
  const lanes = [{ id: 'hh', name: 'Hi-hat', letters: '-xXo+', bars: [{ hits: 'x' + '-'.repeat(15) }] }];
  const onCell = vi.fn();
  render(<Groove view={{ title: '', tempo: 80, bars: [fourFour], lanes }} issues={[]} onCell={onCell} />);

  fireEvent.click(screen.getByRole('button', { name: 'Hi-hat, bar 1, slot 1: x' }));

  expect(onCell).toHaveBeenCalledWith('hh', 0, 0, 'x', '-xXo+');
});
