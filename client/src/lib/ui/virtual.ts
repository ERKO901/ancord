// Jádro virtuálního seznamu s proměnnou výškou řádků. Bez DOM, aby šlo testovat a měřit.

export class HeightIndex {
  private cache = new Map<string, number>();
  offsets: number[] = [0];

  constructor(private estimate: number) {}

  /** Přepočítá offsety pro aktuální klíče. O(n), pro desítky tisíc řádků jednotky ms. */
  rebuild(keys: readonly string[]): void {
    const off = new Array<number>(keys.length + 1);
    off[0] = 0;
    for (let i = 0; i < keys.length; i++) off[i + 1] = (off[i] ?? 0) + (this.cache.get(keys[i] as string) ?? this.estimate);
    this.offsets = off;
  }

  /** Vrátí rozdíl oproti předchozí hodnotě (0, pokud se nezměnila). */
  measure(key: string, height: number): number {
    const prev = this.cache.get(key) ?? this.estimate;
    if (Math.abs(prev - height) < 0.5) return 0;
    this.cache.set(key, height);
    return height - prev;
  }

  get total(): number {
    return this.offsets[this.offsets.length - 1] ?? 0;
  }

  /** Index řádku, který obsahuje svislou pozici `y` (zarovnáno do rozsahu). */
  indexAt(y: number): number {
    const n = this.offsets.length - 1;
    if (n <= 0) return 0;
    let lo = 0, hi = n - 1;
    while (lo < hi) {
      const mid = (lo + hi + 1) >> 1;
      if ((this.offsets[mid] ?? 0) <= y) lo = mid; else hi = mid - 1;
    }
    return lo;
  }

  /** Rozsah [start, end) řádků k vykreslení včetně přesahu v pixelech. */
  range(scrollTop: number, viewport: number, overscan: number): { start: number; end: number } {
    const n = this.offsets.length - 1;
    if (n <= 0) return { start: 0, end: 0 };
    const start = this.indexAt(Math.max(0, scrollTop - overscan));
    const end = Math.min(n, this.indexAt(scrollTop + viewport + overscan) + 1);
    return { start, end };
  }
}
