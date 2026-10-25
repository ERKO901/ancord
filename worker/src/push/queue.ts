import type { Env, PushJob } from "../env";
import { purgeToken, readToken } from "./store";
import { createPushSender, type PushSender } from "./senders";

/** Konzument fronty: odešle tichý push, neplatné tokeny smaže, přechodné chyby zopakuje. */
export async function processPushBatch(
  batch: MessageBatch<PushJob>,
  env: Env,
  sender: PushSender = createPushSender(env),
): Promise<void> {
  for (const msg of batch.messages) {
    const pushId = msg.body?.push_id;
    if (typeof pushId !== "string") {
      msg.ack();
      continue;
    }
    const t = await readToken(env, pushId);
    if (!t) {
      msg.ack();
      continue;
    }
    const result = await sender.send(t.platform, t.token);
    if (result === "sent") msg.ack();
    else if (result === "invalid_token") {
      await purgeToken(env, pushId);
      msg.ack();
    } else msg.retry({ delaySeconds: Math.min(300, 5 * 2 ** Math.min(msg.attempts, 6)) });
  }
}
