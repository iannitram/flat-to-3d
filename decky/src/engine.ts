import { callable } from "@decky/api";
import type {
  CollectionPlan, DoctorReport, GameRow, LaunchText, LinkReport, LinkView,
  LogLines, SettingState, SwitchState, ToastBatch,
} from "./types";

export type ErrorCode = "usage"|"not-built"|"not-found"|"bad-recipe"|"refused"|"link-not-ready"|"command-failed"|"io";
export interface ApiError { code: ErrorCode; message: string }
export type Envelope<T> = { ok: true; command: string; data: T } | { ok: false; command: string; error: ApiError };

const call = callable<[args: string[]], Envelope<unknown>>("engine");

export async function engine<T>(...args: string[]): Promise<Envelope<T>> {
  try {
    return (await call(args)) as Envelope<T>;
  } catch (e) {
    return { ok: false, command: args[0] ?? "", error: { code: "io", message: String(e) } };
  }
}

export const view = () => engine<LinkView>("view");
export const rows = () => engine<GameRow[]>("rows");
export const start3d = () => engine<LinkReport>("start-3d");
export const stop3d = () => engine<LinkReport>("stop-3d");
export const setSwitch = (appid: number, on: boolean) => engine<SwitchState>("switch", "set", String(appid), on ? "on" : "off");
export const setSetting = (key: "auto-3d"|"tunnel", on: boolean) => engine<SettingState>("setting", "set", key, on ? "on" : "off");
export const launch = (verb: "add"|"remove", options: string) => engine<LaunchText>("launch", verb, "--json", "--options", options);
export const doctor = () => engine<DoctorReport>("doctor");
export const collection = (current?: number[]) => engine<CollectionPlan>(...(current ? ["collection", "--current", current.join(",")] : ["collection"]));
export const toasts = (lastSeq: number) => engine<ToastBatch>("toasts", String(lastSeq));
export const logs = (limit: number) => engine<LogLines>("logs", "--limit", String(limit));
