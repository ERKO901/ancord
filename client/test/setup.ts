// jsdom nemá ResizeObserver ani scrollTo na prvcích; stačí minimální náhrady.
class RO {
  constructor(private cb: ResizeObserverCallback) {}
  observe(el: Element) {
    const h = 56;
    queueMicrotask(() => this.cb([{ target: el, borderBoxSize: [{ blockSize: h, inlineSize: 300 }] } as unknown as ResizeObserverEntry], this as unknown as ResizeObserver));
  }
  unobserve() {}
  disconnect() {}
}
(globalThis as unknown as { ResizeObserver: unknown }).ResizeObserver = RO;
Element.prototype.scrollTo = function () {} as typeof Element.prototype.scrollTo;
if (!window.matchMedia) {
  window.matchMedia = ((q: string) => ({ matches: false, media: q, addEventListener() {}, removeEventListener() {} })) as unknown as typeof window.matchMedia;
}
