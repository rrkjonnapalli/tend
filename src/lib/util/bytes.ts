const STEP = 1024;
const UNITS = ['KB', 'MB', 'GB'];

/** 245760 → "240 KB". */
export function bytesLabel(n: number): string {
  if (n < STEP) return `${n} B`;
  let v = n / STEP;
  let i = 0;
  while (v >= STEP && i < UNITS.length - 1) {
    v /= STEP;
    i++;
  }
  // One decimal only for small values past KB ("1.5 MB"), whole numbers otherwise.
  const showDecimal = v < 10 && i > 0;
  const amount = showDecimal ? v.toFixed(1) : Math.round(v);
  return `${amount} ${UNITS[i]}`;
}
