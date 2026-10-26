import { afterEach, expect, test, vi } from 'vitest';
import { linkWriter } from './link';

afterEach(() => {
  vi.useRealTimers();
});

test('writes only the latest link after the wait', () => {
  vi.useFakeTimers();
  const write = vi.fn();
  const { schedule } = linkWriter(write, 250);

  schedule('a');
  schedule('b');
  vi.advanceTimersByTime(249);
  expect(write.mock.calls).toEqual([]);
  vi.advanceTimersByTime(1);
  expect(write.mock.calls).toEqual([['b']]);
});
