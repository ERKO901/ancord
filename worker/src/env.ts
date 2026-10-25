import type { Registry } from "./registry/registry";
import type { RateLimiter } from "./limiter";

export interface Env {
  // bindingy
  REGISTRY: DurableObjectNamespace<Registry>;
  LIMITER: DurableObjectNamespace<RateLimiter>;
  ROUTING_KV: KVNamespace;
  PUSH_KV: KVNamespace;
  BW_BLOBS: R2Bucket;
  PUSH_QUEUE: Queue<PushJob>;

  // proměnné
  PROFILE: string;
  ZONE_DOMAIN: string;
  WORKER_HOST: string;
  CLOUDFLARE_MODE: string; // "real" | "mock"
  CF_ACCOUNT_ID: string;
  CF_ZONE_ID: string;
  NODE_ORIGIN: string;
  GENESIS_NODE_IDS: string;
  PUSH_MODE: string; // "real" | "log"
  CONFIG_OVERRIDES?: string; // JSON, jen pro ladění profilu
  PROOF_ORIGIN?: string; // jen s CLOUDFLARE_MODE=mock: základ URL pro důkaz hostname (lokální node bez tunelu)

  // tajemství
  WORKER_SIGNING_KEY: string;
  CF_API_TOKEN?: string;
  ADMIN_TOKEN?: string;
  CHALLENGE_HMAC_KEY: string;
  PUSH_ENC_KEY: string;
  FCM_SERVICE_ACCOUNT?: string;
  APNS_KEY_P8?: string;
  APNS_KEY_ID?: string;
  APNS_TEAM_ID?: string;
  APNS_TOPIC?: string;
  APNS_HOST?: string;
}

export interface PushJob {
  push_id: string;
}
