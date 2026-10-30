// D5 (ARCHITECTURE.md §13.9): v npm stromu klienta není analytika, telemetrie ani crash reporter.
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { describe, expect, it } from 'vitest';

const FORBIDDEN = /(^|\/)(@sentry\/.*|sentry.*|firebase.*|@firebase\/.*|posthog.*|mixpanel.*|amplitude.*|@segment\/.*|analytics.*|react-ga.*|@datadog\/.*|bugsnag.*|@bugsnag\/.*|logrocket.*|hotjar.*|@opentelemetry\/.*|@tauri-apps\/plugin-(updater|http|shell|fs|opener|log|global-shortcut))$/i;

describe('npm dependency tree', () => {
  it('has no analytics, telemetry, crash reporter or ambient Tauri plugins', () => {
    const lock = JSON.parse(readFileSync(resolve(process.cwd(), 'package-lock.json'), 'utf8'));
    const names = Object.keys(lock.packages ?? {}).filter(Boolean);
    expect(names.length).toBeGreaterThan(10);
    const bad = names.filter((n) => FORBIDDEN.test(n.replace(/^node_modules\//, '').replace(/.*node_modules\//, '')));
    expect(bad).toEqual([]);
  });
});
