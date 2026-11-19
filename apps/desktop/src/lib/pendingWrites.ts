// Everything that still has unsaved edits registers a flush here, so closing the window can
// wait for all of them before the webview goes away.
type Flush = () => Promise<void>;

const flushes = new Set<Flush>();

export function registerFlush(flush: Flush): () => void {
  flushes.add(flush);
  return () => flushes.delete(flush);
}

/** Runs every registered flush; failures are reported but do not stop the others. */
export async function flushAll(): Promise<void> {
  const results = await Promise.allSettled([...flushes].map((f) => f()));
  for (const r of results) {
    if (r.status === "rejected") console.error("failed to save before closing", r.reason);
  }
}
