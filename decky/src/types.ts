export type LinkState = "no-headset" | "unauthorised" | "starting" | "ready" | "stopped";
export type Zone = "green" | "yellow" | "red" | "unknown";

export interface LinkEvent { seq: number; kind: string; text: string }
export interface HealthView { battery: string; temp: string; zone: Zone; colour: string }
export interface LinkView {
  state: LinkState;
  label: string;
  hint: string;
  start: boolean;
  stop: boolean;
  health: HealthView;
  auto_3d: boolean;
  adb_source: string;
}
export interface GameRow {
  appid: number;
  name: string;
  status: string;
  status_label: string;
  status_colour: string;
  switch_on: boolean;
  plays_3d: boolean;
  flat_reason: string | null;
}
export interface LaunchText { options: string }
export interface CollectionPlan { name: string; wanted: number[]; add: number[]; remove: number[] }
export interface ToastBatch { toasts: LinkEvent[]; last_seq: number; title?: string }
export interface LogLines { lines: string[]; text: string }
export interface LinkReport { state: LinkState; message: string }
export interface SwitchState { appid: number; on: boolean }
export interface SettingState { key: string; on: boolean }
export interface DoctorCheck { name: string; ok: boolean; detail: string; fix: string }
export interface DoctorReport { checks: DoctorCheck[]; ok: boolean }
export const TOAST_TITLE = "Flat to 3D";
