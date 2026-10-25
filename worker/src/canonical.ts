import { utf8 } from "./encoding";

export type Json = null | boolean | number | string | Json[] | { [k: string]: Json | undefined };

/** RFC 8785 (JCS) omezené na celá čísla. Viz spec/PROTOCOL.md §1.2. */
export function canonicalJson(value: Json): string {
  if (value === null) return "null";
  switch (typeof value) {
    case "boolean":
      return value ? "true" : "false";
    case "number":
      if (!Number.isSafeInteger(value)) throw new TypeError("canonical JSON allows safe integers only");
      return String(value);
    case "string":
      return JSON.stringify(value);
    case "object": {
      if (Array.isArray(value)) return `[${value.map((v) => canonicalJson(v)).join(",")}]`;
      const keys = Object.keys(value)
        .filter((k) => value[k] !== undefined)
        .sort();
      return `{${keys.map((k) => `${JSON.stringify(k)}:${canonicalJson(value[k] as Json)}`).join(",")}}`;
    }
    default:
      throw new TypeError(`unsupported type in canonical JSON: ${typeof value}`);
  }
}

export function canonicalBytes(value: Json): Uint8Array {
  return utf8(canonicalJson(value));
}
