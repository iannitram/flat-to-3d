import { toaster } from "@decky/api";
import { toasts } from "./engine";
import { TOAST_TITLE } from "./types";

export function startToasts(everyMs = 2000): () => void {
  let lastSeq = 0;
  let busy = false;
  const timer = window.setInterval(async () => {
    if (busy) return;
    busy = true;
    try {
      const b = await toasts(lastSeq);
      if (!b.ok) return;

      for (const t of b.data.toasts) {
        toaster.toast({ title: TOAST_TITLE, body: t.text });
      }
      lastSeq = b.data.last_seq;
    } finally {
      busy = false;
    }
  }, everyMs);

  return () => window.clearInterval(timer);
}
