import { expect, test } from 'vitest';
import { nextLetter } from './cycle';

test.each([
  ['-xXo+', '-', 'x'],
  ['-xXo+', '+', '-'],
  ['-x', 'q', '-'],
])('nextLetter(%s, %s) is %s', (letters, current, next) => {
  expect(nextLetter(letters, current)).toBe(next);
});
