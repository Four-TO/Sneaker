import { writable, get } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";

export type PassthroughMode = "off" | "semi" | "full";
export type Theme = "dark" | "light";
export type BlurMode = "none" | "acrylic" | "mica";

export interface Hotkey {
  toggleShow: string;
  toggleTop: string;
  togglePassthrough: string;
  bossKey: string;
  quickCapture: string;
  toggleChrome: string;
}

export interface Settings {
  theme: Theme;
  opacity: number;
  bgColor: string;
  blur: BlurMode;
  alwaysOnTop: boolean;
  showTitleBar: boolean;
  showTrayIcon: boolean;
  skipTaskbar: boolean;
  passthrough: PassthroughMode;
  autostart: boolean;
  autoLockMinutes: number;
  hasMasterPassword: boolean;
  hotkeys: Hotkey;
  notesDir: string;
  showSidebar: boolean;
  showBottomBar: boolean;
  transparentBg: boolean;
  dragModifier: "Alt" | "Ctrl" | "Shift" | "Meta";
}

export const defaultHotkeys: Hotkey = {
  toggleShow: "Ctrl+Alt+N",
  toggleTop: "Ctrl+Alt+T",
  togglePassthrough: "Ctrl+Alt+P",
  bossKey: "Ctrl+Alt+H",
  quickCapture: "Ctrl+Alt+Q",
  toggleChrome: "Ctrl+Alt+B",
};

export const defaultSettings: Settings = {
  theme: "dark",
  opacity: 0.88,
  bgColor: "#14161e",
  blur: "acrylic",
  alwaysOnTop: false,
  showTitleBar: true,
  showTrayIcon: true,
  skipTaskbar: false,
  passthrough: "off",
  autostart: false,
  autoLockMinutes: 0,
  hasMasterPassword: false,
  hotkeys: { ...defaultHotkeys },
  notesDir: "",
  showSidebar: false,
  showBottomBar: true,
  transparentBg: false,
  dragModifier: "Alt",
};

export const settings = writable<Settings>({ ...defaultSettings });
export const locked = writable<boolean>(false);
export const toast = writable<string>("");

let saveTimer: number | null = null;
export function scheduleSave() {
  if (saveTimer) window.clearTimeout(saveTimer);
  saveTimer = window.setTimeout(async () => {
    try {
      await invoke("save_settings", { settings: get(settings) });
    } catch (e) { console.error(e); }
  }, 300);
}

export function showToast(msg: string, ms = 1600) {
  toast.set(msg);
  window.setTimeout(() => toast.set(""), ms);
}

export async function loadSettings() {
  try {
    const s = await invoke<Settings>("load_settings");
    settings.set({ ...defaultSettings, ...s, hotkeys: { ...defaultSettings.hotkeys, ...(s.hotkeys || {}) } });
  } catch (e) { console.error(e); }
}
