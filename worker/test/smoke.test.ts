import { describe, expect, it } from "vitest";
import { exports } from "cloudflare:workers";

describe("smoke", () => {
  it("health", async () => {
    const res = await exports.default.fetch("https://ancord.test/health");
    expect(res.status).toBe(200);
    const body = (await res.json()) as { ok: boolean; profile: string };
    expect(body.ok).toBe(true);
    expect(body.profile).toBe("dev");
  });

  it("unknown route is a JSON 404", async () => {
    const res = await exports.default.fetch("https://ancord.test/nope");
    expect(res.status).toBe(404);
    expect(((await res.json()) as any).error.code).toBe("not_found");
  });
});
