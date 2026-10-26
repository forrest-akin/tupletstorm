/** The letter after `current` in `letters`, wrapping; an unknown letter gives the first. */
export function nextLetter(letters: string, current: string): string {
  return letters[(letters.indexOf(current) + 1) % letters.length];
}
