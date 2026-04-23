import { invoke } from "@tauri-apps/api/core";
import type { Settings } from "./store";

export interface NoteMeta {
  id: string;
  name: string;
  path: string;
  encrypted: boolean;
  modified: number;
}

export const api = {
  applyWindow: (s: Settings) => invoke("apply_window_settings", { settings: s }),
  applyHotkeys: (s: Settings) => invoke("apply_hotkeys", { settings: s }),
  setOpacity: (v: number) => invoke("set_opacity", { value: v }),
  setAlwaysOnTop: (v: boolean) => invoke("set_always_on_top", { value: v }),
  setPassthrough: (mode: string) => invoke("set_passthrough", { mode }),
  setTitleBar: (v: boolean) => invoke("set_title_bar", { value: v }),
  setTrayVisible: (v: boolean) => invoke("set_tray_visible", { value: v }),
  setSkipTaskbar: (v: boolean) => invoke("set_skip_taskbar", { value: v }),
  setAutostart: (v: boolean) => invoke("set_autostart", { value: v }),
  applyTheme: (s: Settings) => invoke("apply_theme", { settings: s }),

  listNotes: () => invoke<NoteMeta[]>("list_notes"),
  readNote: (id: string, password?: string) => invoke<string>("read_note", { id, password: password || null }),
  saveNote: (id: string, content: string, password?: string) =>
    invoke("save_note", { id, content, password: password || null }),
  createNote: (name: string) => invoke<NoteMeta>("create_note", { name }),
  deleteNote: (id: string) => invoke("delete_note", { id }),
  renameNote: (id: string, name: string) => invoke<NoteMeta>("rename_note", { id, newName: name }),
  importFile: (path: string) => invoke<NoteMeta>("import_file", { path }),
  setNoteEncryption: (id: string, encrypt: boolean, password: string) =>
    invoke("set_note_encryption", { id, encrypt, password }),

  setMasterPassword: (oldPw: string, newPw: string) =>
    invoke("set_master_password", { oldPassword: oldPw, newPassword: newPw }),
  unlock: (pw: string) => invoke<boolean>("unlock_app", { password: pw }),
  lockNow: () => invoke("lock_app"),
  isLocked: () => invoke<boolean>("is_locked"),

  bossHide: () => invoke("boss_hide"),
  toggleMain: () => invoke("toggle_main"),
  pauseHotkeys: () => invoke("pause_hotkeys"),
};
