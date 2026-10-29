import { b64uEncode, utf8 } from "../encoding";
import type { Env } from "../env";

export type Platform = "fcm" | "apns";

export type SendResult = "sent" | "invalid_token" | "retry";

/** Tichý data-push bez obsahu (spec §10): odesílatel nikdy nedostane nic než token. */
export interface PushSender {
  send(platform: Platform, token: string): Promise<SendResult>;
}

/** Záznamový odesílatel pro dev profil: nic neodesílá. */
export class LogPushSender implements PushSender {
  readonly sent: { platform: Platform; token: string }[] = [];
  async send(platform: Platform, token: string): Promise<SendResult> {
    this.sent.push({ platform, token });
    console.log(JSON.stringify({ event: "push_logged", platform }));
    return "sent";
  }
}

function pemToDer(pem: string): Uint8Array {
  const body = pem.replace(/-----(BEGIN|END)[^-]*-----/g, "").replace(/\s+/g, "");
  return Uint8Array.from(atob(body), (c) => c.charCodeAt(0));
}

function b64urJson(o: unknown): string {
  return b64uEncode(utf8(JSON.stringify(o)));
}

interface CachedToken {
  value: string;
  expiresAt: number;
}

export class RealPushSender implements PushSender {
  private fcmToken: CachedToken | null = null;
  private apnsToken: CachedToken | null = null;
  private readonly fetcher: typeof fetch;

  constructor(
    private readonly env: Env,
    fetcher: typeof fetch = fetch,
    private readonly nowMs: () => number = Date.now,
  ) {
    // globální fetch nesmí být volaný jako metoda objektu (workerd: „Illegal invocation“)
    this.fetcher = (input, init) => fetcher(input, init);
  }

  async send(platform: Platform, token: string): Promise<SendResult> {
    try {
      return platform === "fcm" ? await this.sendFcm(token) : await this.sendApns(token);
    } catch (e) {
      console.log(JSON.stringify({ event: "push_error", platform, error: String(e) }));
      return "retry";
    }
  }

  private async fcmAccessToken(sa: { client_email: string; private_key: string; token_uri?: string }): Promise<string> {
    if (this.fcmToken && this.fcmToken.expiresAt > this.nowMs() + 60_000) return this.fcmToken.value;
    const iat = Math.floor(this.nowMs() / 1000);
    const uri = sa.token_uri ?? "https://oauth2.googleapis.com/token";
    const unsigned = `${b64urJson({ alg: "RS256", typ: "JWT" })}.${b64urJson({
      iss: sa.client_email,
      scope: "https://www.googleapis.com/auth/firebase.messaging",
      aud: uri,
      iat,
      exp: iat + 3600,
    })}`;
    const key = await crypto.subtle.importKey(
      "pkcs8",
      pemToDer(sa.private_key),
      { name: "RSASSA-PKCS1-v1_5", hash: "SHA-256" },
      false,
      ["sign"],
    );
    const sig = new Uint8Array(await crypto.subtle.sign("RSASSA-PKCS1-v1_5", key, utf8(unsigned)));
    const res = await this.fetcher(uri, {
      method: "POST",
      headers: { "content-type": "application/x-www-form-urlencoded" },
      body: new URLSearchParams({
        grant_type: "urn:ietf:params:oauth:grant-type:jwt-bearer",
        assertion: `${unsigned}.${b64uEncode(sig)}`,
      }),
    });
    if (!res.ok) throw new Error(`FCM OAuth ${res.status}`);
    const body = (await res.json()) as { access_token: string; expires_in: number };
    this.fcmToken = { value: body.access_token, expiresAt: this.nowMs() + body.expires_in * 1000 };
    return body.access_token;
  }

  private async sendFcm(token: string): Promise<SendResult> {
    if (!this.env.FCM_SERVICE_ACCOUNT) throw new Error("FCM_SERVICE_ACCOUNT is not configured");
    const sa = JSON.parse(this.env.FCM_SERVICE_ACCOUNT) as { project_id: string; client_email: string; private_key: string };
    const access = await this.fcmAccessToken(sa);
    const res = await this.fetcher(`https://fcm.googleapis.com/v1/projects/${sa.project_id}/messages:send`, {
      method: "POST",
      headers: { authorization: `Bearer ${access}`, "content-type": "application/json" },
      body: JSON.stringify({
        message: {
          token,
          data: { t: "wake" },
          android: { priority: "HIGH" },
        },
      }),
    });
    if (res.ok) return "sent";
    if (res.status === 404 || res.status === 400) {
      const text = await res.text();
      if (/UNREGISTERED|INVALID_ARGUMENT|NOT_FOUND/.test(text)) return "invalid_token";
    }
    if (res.status === 401) this.fcmToken = null;
    return "retry";
  }

  private async apnsJwt(): Promise<string> {
    if (this.apnsToken && this.apnsToken.expiresAt > this.nowMs() + 60_000) return this.apnsToken.value;
    const { APNS_KEY_P8, APNS_KEY_ID, APNS_TEAM_ID } = this.env;
    if (!APNS_KEY_P8 || !APNS_KEY_ID || !APNS_TEAM_ID) throw new Error("APNs is not configured");
    const iat = Math.floor(this.nowMs() / 1000);
    const unsigned = `${b64urJson({ alg: "ES256", kid: APNS_KEY_ID })}.${b64urJson({ iss: APNS_TEAM_ID, iat })}`;
    const key = await crypto.subtle.importKey("pkcs8", pemToDer(APNS_KEY_P8), { name: "ECDSA", namedCurve: "P-256" }, false, ["sign"]);
    // WebCrypto vrací podpis ECDSA ve formátu r‖s, což je přesně to, co JWS ES256 vyžaduje
    const sig = new Uint8Array(await crypto.subtle.sign({ name: "ECDSA", hash: "SHA-256" }, key, utf8(unsigned)));
    const jwt = `${unsigned}.${b64uEncode(sig)}`;
    this.apnsToken = { value: jwt, expiresAt: this.nowMs() + 45 * 60_000 };
    return jwt;
  }

  private async sendApns(token: string): Promise<SendResult> {
    if (!/^[0-9a-fA-F]{32,200}$/.test(token)) return "invalid_token";
    if (!this.env.APNS_TOPIC) throw new Error("APNS_TOPIC is not configured");
    const host = this.env.APNS_HOST ?? "api.push.apple.com";
    const res = await this.fetcher(`https://${host}/3/device/${token}`, {
      method: "POST",
      headers: {
        authorization: `bearer ${await this.apnsJwt()}`,
        "apns-topic": this.env.APNS_TOPIC,
        "apns-push-type": "background",
        "apns-priority": "5",
        "content-type": "application/json",
      },
      body: JSON.stringify({ aps: { "content-available": 1 } }),
    });
    if (res.ok) return "sent";
    if (res.status === 410) return "invalid_token";
    if (res.status === 400) {
      const text = await res.text();
      if (/BadDeviceToken|DeviceTokenNotForTopic|Unregistered/.test(text)) return "invalid_token";
    }
    if (res.status === 403) this.apnsToken = null;
    return "retry";
  }
}

export function createPushSender(env: Env): PushSender {
  // log mimo profil dev by push v produkci potichu zahazoval
  if (env.PROFILE !== "dev" && env.PUSH_MODE !== "real") {
    throw new Error(`PUSH_MODE must be 'real' outside the dev profile (PROFILE=${env.PROFILE})`);
  }
  return env.PUSH_MODE === "real" ? new RealPushSender(env) : new LogPushSender();
}
