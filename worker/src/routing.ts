import { randomFloat } from "./encoding";

export interface SnapshotNode {
  id: string;
  host: string;
  /** základní skóre (capacity × health × trust) */
  b: number;
  /** skóre po odečtení zátěže */
  s: number;
  cc: string | null;
  ct: string | null;
  lat: number | null;
  lon: number | null;
}

export interface Snapshot {
  v: 1;
  generated_at: number;
  epoch: number;
  nodes: SnapshotNode[];
}

export interface ClientGeo {
  lat: number | null;
  lon: number | null;
  country: string | null;
  continent: string | null;
}

export function haversineKm(lat1: number, lon1: number, lat2: number, lon2: number): number {
  const rad = Math.PI / 180;
  const dLat = (lat2 - lat1) * rad;
  const dLon = (lon2 - lon1) * rad;
  const a = Math.sin(dLat / 2) ** 2 + Math.cos(lat1 * rad) * Math.cos(lat2 * rad) * Math.sin(dLon / 2) ** 2;
  return 6371 * 2 * Math.asin(Math.min(1, Math.sqrt(a)));
}

function distanceKm(client: ClientGeo, n: SnapshotNode): number {
  if (client.lat === null || client.lon === null || n.lat === null || n.lon === null) return Infinity;
  return haversineKm(client.lat, client.lon, n.lat, n.lon);
}

function weightedIndex(weights: number[], rng: () => number): number {
  let total = 0;
  for (const w of weights) total += w;
  if (total <= 0) return Math.floor(rng() * weights.length);
  let x = rng() * total;
  for (let i = 0; i < weights.length; i++) {
    x -= weights[i]!;
    if (x < 0) return i;
  }
  return weights.length - 1;
}

/** Power-of-two-choices: dva kandidáti vážení základním skóre, vyhrává ten s nižší zátěží (vyšší `s`). */
function pickP2C(remaining: SnapshotNode[], rng: () => number): number {
  if (remaining.length === 1) return 0;
  const weights = remaining.map((n) => n.b);
  const a = weightedIndex(weights, rng);
  const w2 = weights.slice();
  w2[a] = 0;
  const hasOther = w2.some((w) => w > 0);
  let b: number;
  if (hasOther) b = weightedIndex(w2, rng);
  else {
    b = Math.floor(rng() * (remaining.length - 1));
    if (b >= a) b++;
  }
  return remaining[a]!.s >= remaining[b]!.s ? a : b;
}

/**
 * Výběr `count` nodů pro klienta (ARCHITECTURE §5.3):
 * 1) kandidáti se základním skóre > 0, 2) filtr podle vzdálenosti/regionu klienta, 3) P2C bez opakování.
 * Při `count` a méně kandidátech jde jen o failover seřazený podle skóre.
 */
export function selectNodes(
  nodes: SnapshotNode[],
  client: ClientGeo,
  count: number,
  nearKm: number,
  rng: () => number = randomFloat,
): SnapshotNode[] {
  const candidates = nodes.filter((n) => n.b > 0);
  if (candidates.length <= count) return candidates.slice().sort((a, b) => b.s - a.s || (a.id < b.id ? -1 : 1));

  const chosen: SnapshotNode[] = [];
  let pool: SnapshotNode[];
  const hasGeo = client.lat !== null && client.lon !== null;
  if (hasGeo) {
    const withDist = candidates.map((n) => ({ n, d: distanceKm(client, n) })).sort((x, y) => x.d - y.d);
    const near = withDist.filter((x) => x.d <= nearKm).map((x) => x.n);
    if (near.length >= count) pool = near;
    else {
      // všichni nodé v dosahu jsou jistí, zbytek se doplní z nejbližších dalších
      chosen.push(...near.sort((a, b) => b.s - a.s));
      const rest = withDist.slice(near.length).map((x) => x.n);
      pool = rest.slice(0, (count - near.length) * 2);
    }
  } else if (client.continent) {
    const same = candidates.filter((n) => n.ct === client.continent);
    pool = same.length >= count ? same : candidates;
  } else {
    pool = candidates;
  }

  const remaining = pool.slice();
  while (chosen.length < count && remaining.length > 0) {
    chosen.push(remaining.splice(pickP2C(remaining, rng), 1)[0]!);
  }
  if (chosen.length < count) {
    const used = new Set(chosen.map((n) => n.id));
    const rest = candidates.filter((n) => !used.has(n.id));
    while (chosen.length < count && rest.length > 0) chosen.push(rest.splice(pickP2C(rest, rng), 1)[0]!);
  }
  return chosen;
}
