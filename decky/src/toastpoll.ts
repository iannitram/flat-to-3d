import type { Envelope } from "./engine";
import type { ToastBatch } from "./types";

export interface ToastPoller {
  tick(): Promise<void>;
  stop(): void;
}

export function createToastPoller(
  fetchBatch: (lastSeq: number) => Promise<Envelope<ToastBatch>>,
  show: (text: string) => void,
): ToastPoller {
  let lastSeq: number | null = null;
  let busy = false;
  let stopped = false;

  return {
    async tick() {
      if (busy || stopped) return;
      busy = true;
      try {
        const batch = await fetchBatch(lastSeq ?? 0);
        if (!batch.ok || stopped) return;
        if (lastSeq === null) {
          lastSeq = batch.data.last_seq;
          return;
        }
        for (const toast of batch.data.toasts) show(toast.text);
        lastSeq = batch.data.last_seq;
      } finally {
        busy = false;
      }
    },
    stop() {
      stopped = true;
    },
  };
}
