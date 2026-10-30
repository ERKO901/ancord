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
const issueLabel = {
  worker_unreachable: "Offline: server nedostupný nebo bez sítě",
  worker_error: "Server hlásí chybu",
  route_untrusted: "Trasa se nepodařilo ověřit",
  node_unreachable: "Offline: žádný uzel neodpovídá",
  node_rejected: "Uzel odmítl požadavek (nejspíš starší verze)",
  protocol_mismatch: "Uzly mají jinou verzi protokolu, aktualizujte aplikaci",
} as const;

/** Text stavu spojení; v offline přidá důvod posledního selhání. */
export function connText(conn: keyof typeof connLabel, issue: keyof typeof issueLabel | null): string {
  return conn === "offline" && issue ? issueLabel[issue] : connLabel[conn];
}
export const stateLabel = {
  queued: "Čeká na odeslání", pending: "Odesílá se", delivered: "Doručeno", unknown: "Stav neznámý",
} as const;
