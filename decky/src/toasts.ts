import { toaster } from "@decky/api";
import { toasts } from "./engine";
import { TOAST_TITLE } from "./types";
import { createToastPoller } from "./toastpoll";

export function startToasts(everyMs = 2000): () => void {
  const poller = createToastPoller(toasts, (text) => {
    toaster.toast({ title: TOAST_TITLE, body: text });
  });
  const tick = () => poller.tick().catch((e) => console.error("Flat to 3D: toast poll failed", e));
  void tick();
  const timer = window.setInterval(() => {
    void tick();
  }, everyMs);

  return () => {
    poller.stop();
    window.clearInterval(timer);
  };
}
