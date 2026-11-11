import { collection, type ApiError } from "./engine";
import type { CollectionPlan } from "./types";

let name = "3D Ready";

export async function syncCollection(): Promise<CollectionPlan | ApiError> {
  const id = collectionStore.GetCollectionIDByUserTag(name);
  const c =
    (id ? collectionStore.GetCollection(id) : null) ??
    collectionStore.NewUnsavedCollection(
      name,
      undefined as unknown as Parameters<
        typeof collectionStore.NewUnsavedCollection
      >[1],
      [],
    );
  const r = await collection(Array.from(c.apps.keys()));
  if (!r.ok) return r.error;

  name = r.data.name;
  const ov = (ids: number[]) =>
    ids
      .map((i) => appStore.GetAppOverviewByAppID(i))
      .filter((o): o is NonNullable<typeof o> => !!o);

  if (r.data.add.length + r.data.remove.length > 0) {
    c.AddApps(ov(r.data.add));
    c.RemoveApps(ov(r.data.remove));
    await c.Save();
  }
  return r.data;
}
