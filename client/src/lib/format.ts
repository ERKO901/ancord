const cs = "cs-CZ";

export function timeShort(ts: number, now = Date.now()): string {
  const d = new Date(ts);
  const n = new Date(now);
  if (d.toDateString() === n.toDateString()) return d.toLocaleTimeString(cs, { hour: "2-digit", minute: "2-digit" });
  const days = Math.floor((now - ts) / 86_400_000);
  if (days < 7) return d.toLocaleDateString(cs, { weekday: "short" });
  return d.toLocaleDateString(cs, { day: "numeric", month: "numeric" });
}

export function timeFull(ts: number): string {
  return new Date(ts).toLocaleString(cs, { dateStyle: "medium", timeStyle: "short" });
}

export function initials(name: string): string {
  const parts = name.trim().split(/\s+/).filter(Boolean);
  return ((parts[0]?.[0] ?? "?") + (parts.length > 1 ? (parts[parts.length - 1]?.[0] ?? "") : "")).toUpperCase();
}

export function shortAddress(a: string): string {
  const s = a.replace(/^ancord:/, "");
  return s.length > 16 ? `${s.slice(0, 8)}…${s.slice(-6)}` : s;
}

export const connLabel = { offline: "Offline", connecting: "Připojování…", online: "Online" } as const;
export const stateLabel = {
  queued: "Čeká na odeslání", pending: "Odesílá se", delivered: "Doručeno", unknown: "Stav neznámý",
} as const;
