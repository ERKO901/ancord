/** Čas Workeru. Testy ho mohou přepsat; produkční kód používá vždy tento objekt, ne `Date.now()`. */
export const clock = {
  nowMs: (): number => Date.now(),
  nowS(): number {
    return Math.floor(this.nowMs() / 1000);
  },
};
