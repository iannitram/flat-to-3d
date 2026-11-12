import {
  ButtonItem,
  Field,
  PanelSection,
  PanelSectionRow,
  ToggleField,
} from "@decky/ui";
import { useEffect, useRef, useState, type JSX } from "react";
import {
  doctor,
  rows,
  setSetting,
  setSwitch,
  start3d,
  stop3d,
  view,
  type ApiError,
} from "./engine";
import type { DoctorReport, GameRow, LinkView } from "./types";
import { syncCollection } from "./collection";
import { syncLaunchOptions } from "./launch";

export function Content(): JSX.Element {
  const [v, setView] = useState<LinkView | null>(null);
  const [games, setGames] = useState<GameRow[]>([]);
  const [errors, setErrors] = useState<ApiError[]>([]);
  const [pollErrors, setPollErrors] = useState<ApiError[]>([]);
  const [report, setReport] = useState<DoctorReport | null>(null);

  const mounted = useRef(true);
  const polling = useRef<Promise<void> | null>(null);

  // One view+rows poll at a time: a slow engine call must not stack polls
  // whose answers land out of order. An action's refresh waits for the poll
  // that is out, then polls again so it sees the action's result.
  const poll = async () => {
    const [viewResult, rowsResult] = await Promise.all([view(), rows()]);
    if (!mounted.current) return;
    const nextErrors: ApiError[] = [];
    if (viewResult.ok) setView(viewResult.data);
    else nextErrors.push(viewResult.error);
    if (rowsResult.ok) setGames(rowsResult.data);
    else nextErrors.push(rowsResult.error);
    setPollErrors(nextErrors);
  };

  const refresh = async (again = false) => {
    if (polling.current) {
      await polling.current;
      if (!again) return;
    }
    if (!mounted.current || polling.current) return;
    const p = poll().finally(() => {
      polling.current = null;
    });
    polling.current = p;
    await p;
  };

  useEffect(() => {
    mounted.current = true;
    void refresh();
    const timer = window.setInterval(() => void refresh(), 1000);
    void syncCollection()
      .then((plan) =>
        "code" in plan ? [plan] : syncLaunchOptions(plan),
      )
      // A Steam client call that throws: kept as "io", as engine.ts does.
      .catch((e): ApiError[] => [{ code: "io", message: String(e) }])
      .then((syncErrors) => {
        if (mounted.current) setErrors((current) => [...current, ...syncErrors]);
      });
    return () => {
      mounted.current = false;
      window.clearInterval(timer);
    };
  }, []);

  const runSetting = async (key: "auto-3d", on: boolean) => {
    const result = await setSetting(key, on);
    if (!mounted.current) return;
    if (!result.ok) setErrors((current) => [...current, result.error]);
    await refresh(true);
  };

  const runSwitch = async (row: GameRow, on: boolean) => {
    const result = await setSwitch(row.appid, on);
    if (!mounted.current) return;
    if (!result.ok) setErrors((current) => [...current, result.error]);
    await refresh(true);
  };

  const runLinkAction = async (action: typeof start3d | typeof stop3d) => {
    const result = await action();
    if (!mounted.current) return;
    if (!result.ok) setErrors((current) => [...current, result.error]);
    await refresh(true);
  };

  const checkSetup = async () => {
    const result = await doctor();
    if (!mounted.current) return;
    if (!result.ok) {
      setErrors((current) => [...current, result.error]);
      return;
    }
    setReport(result.data);
  };

  return (
    <PanelSection>
      {v ? (
        <Field label={v.label} description={v.hint} />
      ) : (
        <Field label="Flat to 3D" description="Loading…" />
      )}
      {[...pollErrors, ...errors].map((error, i) => (
        <Field
          key={`${error.code}-${i}`}
          label={error.code}
          description={error.message}
        />
      ))}
      {v && (
        <>
          <PanelSectionRow>
            <ButtonItem
              disabled={!v.start}
              onClick={() => void runLinkAction(start3d)}
            >
              Start 3D
            </ButtonItem>
          </PanelSectionRow>
          <PanelSectionRow>
            <ButtonItem
              disabled={!v.stop}
              onClick={() => void runLinkAction(stop3d)}
            >
              Stop 3D
            </ButtonItem>
          </PanelSectionRow>
          <Field label="Battery" description={v.health.battery} />
          <PanelSectionRow>
            <div style={{ color: v.health.colour }}>
              <Field label="Temperature" description={v.health.temp} />
            </div>
          </PanelSectionRow>
          <PanelSectionRow>
            <ToggleField
              label="Auto 3D"
              checked={v.auto_3d}
              onChange={(on) => void runSetting("auto-3d", on)}
            />
          </PanelSectionRow>
          {games.map((row) => (
            <PanelSectionRow key={row.appid}>
              <ToggleField
                label={row.name}
                description={row.flat_reason ?? row.status_label}
                checked={row.switch_on}
                onChange={(on) => void runSwitch(row, on)}
              />
            </PanelSectionRow>
          ))}
          <PanelSectionRow>
            <ButtonItem onClick={() => void checkSetup()}>Check setup</ButtonItem>
          </PanelSectionRow>
          {report?.checks
            .filter((check) => !check.ok)
            .map(({ name, detail, fix }) => (
              <Field
                key={name}
                label={name}
                description={`${detail} — ${fix}`}
              />
            ))}
        </>
      )}
    </PanelSection>
  );
}
