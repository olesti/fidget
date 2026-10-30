import { invoke } from "@tauri-apps/api/core";

export type Field = {
  key: string;
  label: string;
  kind: "number" | "select" | "bool" | "time";
  value: string | number | boolean;
  options?: string[];
};
export type Feature = { id: string; name: string; enabled: boolean; fields: Field[]; shortcut: string };
export type State = {
  paused: boolean;
  lang: "en" | "tr";
  strings: Record<string, string>;
  autostart: boolean;
  version: string;
  features: Feature[];
};
export type Permission = { id: string; granted: boolean };

export const api = {
  state: () => invoke<State>("get_state"),
  setEnabled: (id: string, enabled: boolean) => invoke<boolean>("set_enabled", { id, enabled }),
  setConfig: (id: string, config: Record<string, unknown>) => invoke<void>("set_config", { id, config }),
  setLang: (lang: string) => invoke<void>("set_lang", { lang }),
  setAutostart: (enabled: boolean) => invoke<void>("set_autostart", { enabled }),
  setShortcut: (id: string, accel: string) => invoke<void>("set_shortcut", { id, accel }),
  permissions: () => invoke<Permission[]>("get_permissions"),
  requestPermission: (id: string) => invoke<void>("request_permission", { id }),
  openLogs: () => invoke<void>("open_logs"),
};
