import { launch, type ApiError } from "./engine";
import type { CollectionPlan } from "./types";

export async function syncLaunchOptions(
  plan: CollectionPlan,
): Promise<ApiError[]> {
  const errors: ApiError[] = [];

  for (const id of plan.wanted) {
    await appDetailsStore.RequestAppDetails(id);
    const cur = appDetailsStore.GetAppDetails(id)?.strLaunchOptions ?? "";
    const r = await launch("add", cur);
    if (!r.ok) {
      errors.push(r.error);
    } else if (r.data.options !== cur) {
      SteamClient.Apps.SetAppLaunchOptions(id, r.data.options);
    }
  }

  for (const id of plan.remove) {
    await appDetailsStore.RequestAppDetails(id);
    const cur = appDetailsStore.GetAppDetails(id)?.strLaunchOptions ?? "";
    const r = await launch("remove", cur);
    if (!r.ok) {
      errors.push(r.error);
    } else if (r.data.options !== cur) {
      SteamClient.Apps.SetAppLaunchOptions(id, r.data.options);
    }
  }

  return errors;
}
