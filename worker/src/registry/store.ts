export type NodeStatus = "provisioning" | "pending" | "active" | "retired" | "banned";

export interface Claimed {
  cpu_cores: number;
  ram_mb: number;
  storage_gb: number;
  bandwidth_mbps: number;
}

export interface Load {
  conns: number;
  max_conns: number;
  cpu_pct: number;
  mem_pct: number;
}

export interface NodeRow {
  node_id: string;
  pubkey: string;
  hostname: string;
  status: NodeStatus;
  genesis: number;
  claimed: string; // JSON Claimed
  asn: number | null;
  ip_hash: string | null;
  tunnel_id: string | null;
  tunnel_name: string | null;
  dns_record_id: string | null;
  created_at: number;
  updated_at: number;
  activated_at: number | null;
  expires_at: number | null;
  last_hb_at: number | null;
  hb_seq: number;
  load: string | null; // JSON Load
  country: string | null;
  continent: string | null;
  colo: string | null;
  lat: number | null;
  lon: number | null;
  rtt_ms: number | null;
  m_cpu_hps: number | null;
  m_ram_mb: number | null;
  m_storage_gb: number | null;
  m_bw_mbps: number | null;
  m_bw_r2: number | null;
  m_bw_wit: number | null;
  health: number;
  strikes: number;
  ch_ok: number;
  ch_total: number;
  uptime_s: number;
  ban_reason: string | null;
}

const MIGRATIONS: string[] = [
  `
  CREATE TABLE nodes (
    node_id TEXT PRIMARY KEY,
    pubkey TEXT NOT NULL UNIQUE,
    hostname TEXT NOT NULL UNIQUE,
    status TEXT NOT NULL,
    genesis INTEGER NOT NULL DEFAULT 0,
    claimed TEXT NOT NULL,
    asn INTEGER,
    ip_hash TEXT,
    tunnel_id TEXT,
    tunnel_name TEXT,
    dns_record_id TEXT,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    activated_at INTEGER,
    expires_at INTEGER,
    last_hb_at INTEGER,
    hb_seq INTEGER NOT NULL DEFAULT 0,
    load TEXT,
    country TEXT, continent TEXT, colo TEXT, lat REAL, lon REAL,
    rtt_ms REAL,
    m_cpu_hps REAL, m_ram_mb REAL, m_storage_gb REAL, m_bw_mbps REAL, m_bw_r2 REAL, m_bw_wit REAL,
    health REAL NOT NULL DEFAULT 1,
    strikes INTEGER NOT NULL DEFAULT 0,
    ch_ok INTEGER NOT NULL DEFAULT 0,
    ch_total INTEGER NOT NULL DEFAULT 0,
    uptime_s INTEGER NOT NULL DEFAULT 0,
    ban_reason TEXT
  );
  CREATE INDEX nodes_status ON nodes (status);
  CREATE INDEX nodes_asn ON nodes (asn);
  CREATE INDEX nodes_ip ON nodes (ip_hash);

  CREATE TABLE reg_challenges (
    nonce TEXT PRIMARY KEY,
    pubkey TEXT NOT NULL,
    hostname TEXT NOT NULL,
    claimed TEXT NOT NULL,
    hashcash_bits INTEGER NOT NULL,
    asn INTEGER,
    expires_at INTEGER NOT NULL
  );
  CREATE INDEX reg_challenges_pubkey ON reg_challenges (pubkey);
  CREATE INDEX reg_challenges_exp ON reg_challenges (expires_at);

  CREATE TABLE reg_log (at INTEGER NOT NULL);
  CREATE INDEX reg_log_at ON reg_log (at);

  CREATE TABLE nonces (key TEXT PRIMARY KEY, expires_at INTEGER NOT NULL);
  CREATE INDEX nonces_exp ON nonces (expires_at);

  CREATE TABLE directories (
    epoch INTEGER PRIMARY KEY,
    body TEXT NOT NULL,
    sig TEXT NOT NULL,
    kid TEXT NOT NULL
  );

  CREATE TABLE schedule (
    node_id TEXT NOT NULL,
    kind TEXT NOT NULL,
    next_due INTEGER NOT NULL,
    strikes INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (node_id, kind)
  );

  CREATE TABLE challenges (
    id TEXT PRIMARY KEY,
    node_id TEXT NOT NULL,
    kind TEXT NOT NULL,
    state TEXT NOT NULL,
    params TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    sent_at INTEGER,
    deadline_at INTEGER NOT NULL,
    rtt_ms REAL,
    data TEXT
  );
  CREATE INDEX challenges_node ON challenges (node_id, state);
  CREATE INDEX challenges_deadline ON challenges (deadline_at);

  CREATE TABLE space (
    node_id TEXT NOT NULL,
    tier TEXT NOT NULL,
    seed TEXT NOT NULL,
    root TEXT NOT NULL,
    leaves INTEGER NOT NULL,
    committed_at INTEGER NOT NULL,
    PRIMARY KEY (node_id, tier)
  );

  CREATE TABLE asn_penalties (asn INTEGER PRIMARY KEY, count INTEGER NOT NULL);

  CREATE TABLE meta (key TEXT PRIMARY KEY, value TEXT NOT NULL);
  `,
];

export function migrate(sql: SqlStorage): void {
  sql.exec("CREATE TABLE IF NOT EXISTS schema_version (v INTEGER NOT NULL)");
  const row = sql.exec<{ v: number }>("SELECT MAX(v) AS v FROM schema_version").toArray()[0];
  const current = row?.v ?? 0;
  for (let i = current; i < MIGRATIONS.length; i++) {
    sql.exec(MIGRATIONS[i]!);
    sql.exec("INSERT INTO schema_version (v) VALUES (?)", i + 1);
  }
}

export function all<T extends Record<string, SqlStorageValue>>(sql: SqlStorage, query: string, ...b: SqlStorageValue[]): T[] {
  return sql.exec<T>(query, ...b).toArray();
}

export function one<T extends Record<string, SqlStorageValue>>(sql: SqlStorage, query: string, ...b: SqlStorageValue[]): T | null {
  return sql.exec<T>(query, ...b).toArray()[0] ?? null;
}

export function run(sql: SqlStorage, query: string, ...b: SqlStorageValue[]): void {
  sql.exec(query, ...b);
}

export function getNode(sql: SqlStorage, nodeId: string): NodeRow | null {
  return one<NodeRow & Record<string, SqlStorageValue>>(sql, "SELECT * FROM nodes WHERE node_id = ?", nodeId);
}

export function getMeta(sql: SqlStorage, key: string): string | null {
  return one<{ value: string }>(sql, "SELECT value FROM meta WHERE key = ?", key)?.value ?? null;
}

export function setMeta(sql: SqlStorage, key: string, value: string): void {
  sql.exec("INSERT INTO meta (key, value) VALUES (?, ?) ON CONFLICT(key) DO UPDATE SET value = excluded.value", key, value);
}
