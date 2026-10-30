import { describe, expect, it } from "vitest";
import { HeightIndex } from "../src/lib/ui/virtual";

const keys = (n: number) => Array.from({ length: n }, (_, i) => `k${i}`);

describe("HeightIndex", () => {
  it("odhaduje výšku, dokud řádek není změřený", () => {
    const h = new HeightIndex(50);
    h.rebuild(keys(4));
    expect(h.total).toBe(200);
    expect(h.measure("k1", 100)).toBe(50);
    h.rebuild(keys(4));
    expect(h.total).toBe(250);
    expect(h.offsets).toEqual([0, 50, 150, 200, 250]);
  });

  it("neoznačí změnu menší než půl pixelu", () => {
    const h = new HeightIndex(50);
    expect(h.measure("a", 50.2)).toBe(0);
  });

  it("najde řádek podle pozice a vrátí okno s přesahem", () => {
    const h = new HeightIndex(10);
    h.rebuild(keys(100));
    expect(h.indexAt(0)).toBe(0);
    expect(h.indexAt(95)).toBe(9);
    expect(h.indexAt(99999)).toBe(99);
    expect(h.range(500, 100, 0)).toEqual({ start: 50, end: 61 });
    expect(h.range(0, 100, 50)).toEqual({ start: 0, end: 16 });
  });

  it("prázdný seznam má prázdné okno", () => {
    const h = new HeightIndex(10);
    h.rebuild([]);
    expect(h.range(0, 500, 100)).toEqual({ start: 0, end: 0 });
  });

  it("10 000 řádků se přepočítá a prohledá rychle (rozpočet §13.11)", () => {
    const h = new HeightIndex(56);
    const ks = keys(10_000);
    const t0 = performance.now();
    h.rebuild(ks);
    const r = h.range(h.total - 800, 800, 600);
    const dt = performance.now() - t0;
    expect(r.end - r.start).toBeLessThan(60);
    expect(dt).toBeLessThan(50);
  });
});
