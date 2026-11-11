import { launch, type ApiError } from "./engine";
import type { CollectionPlan } from "./types";

// The load-time sync and the panel's mount sync share one collection plan
// (syncCollection de-duplicates its in-flight run); share the launch pass too,
// so two passes never read and write the same app's options at once.
let inflight: { plan: CollectionPlan; done: Promise<ApiError[]> } | null = null;

export function syncLaunchOptions(plan: CollectionPlan): Promise<ApiError[]> {
  if (inflight?.plan !== plan) {
    const done = runSync(plan).finally(() => {
      if (inflight?.done === done) inflight = null;
    });
    inflight = { plan, done };
  }
  return inflight.done;
}

async function runSync(plan: CollectionPlan): Promise<ApiError[]> {
  const errors: ApiError[] = [];
  for (const id of plan.wanted) await syncOne(id, "add", errors);
  for (const id of plan.remove) await syncOne(id, "remove", errors);
  return errors;
}

// One app. A Steam client call that throws is kept as an "io" error, the same
// way engine.ts keeps a failed backend call, and the next app still runs.
async function syncOne(
  id: number,
  verb: "add" | "remove",
  errors: ApiError[],
): Promise<void> {
  const { appDetailsStore, SteamClient } = window;
  try {
    await appDetailsStore.RequestAppDetails(id);
    const cur = appDetailsStore.GetAppDetails(id)?.strLaunchOptions ?? "";
    const r = await launch(verb, cur);
    if (!r.ok) {
      errors.push(r.error);
    } else if (r.data.options !== cur) {
      SteamClient.Apps.SetAppLaunchOptions(id, r.data.options);
    }
  } catch (e) {
    errors.push({ code: "io", message: String(e) });
  }
}
