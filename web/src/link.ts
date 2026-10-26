/** Writes only the last scheduled link, once `waitMs` pass without another. */
export function linkWriter(write: (link: string) => void, waitMs: number) {
  let timer: ReturnType<typeof setTimeout> | undefined;
  return {
    schedule: (link: string) => {
      clearTimeout(timer);
      timer = setTimeout(() => write(link), waitMs);
    },
  };
}
