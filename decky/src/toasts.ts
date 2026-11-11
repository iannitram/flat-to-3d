import { toaster } from "@decky/api";
import { toasts } from "./engine";
import { TOAST_TITLE } from "./types";

export function startToasts(everyMs = 2000): () => void {
  let lastSeq = 0;
  let busy = false;
  let stopped = false;
  const tick = async () => {
    if (busy || stopped) return;
    busy = true;
    try {
      const b = await toasts(lastSeq);
      // stop() ran while the call was out: show nothing after unload.
      if (!b.ok || stopped) return;

      for (const t of b.data.toasts) {
        toaster.toast({ title: TOAST_TITLE, body: t.text });
      }
      lastSeq = b.data.last_seq;
    } finally {
      busy = false;
    }
  };
  const timer = window.setInterval(() => {
    tick().catch((e) => console.error("Flat to 3D: toast poll failed", e));
  }, everyMs);

  return () => {
    stopped = true;
    window.clearInterval(timer);
  };
}
