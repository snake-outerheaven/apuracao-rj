// SPDX-License-Identifier: GPL-3.0-or-later
// Abstração de invoke/listen do Tauri; no navegador cai para um mock em memória.
import { mockHistorico, mockSnapshotAt, PASSOS_MOCK } from "./mock";
import type { PontoHistorico, Settings, Snapshot } from "./types";

export type Unlisten = () => void;
export type Api = {
  readonly modo: "tauri" | "navegador";
  getSnapshot(): Promise<Snapshot>;
  refreshNow(): Promise<Snapshot>;
  getHistory(): Promise<PontoHistorico[]>;
  getSettings(): Promise<Settings>;
  setSettings(s: Settings): Promise<void>;
  onUpdate(cb: (s: Snapshot) => void): Promise<Unlisten>;
  onError(cb: (msg: string) => void): Promise<Unlisten>;
};

export const noTauri = (): boolean => typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

function tauriApi(): Api {
  const core = () => import("@tauri-apps/api/core");
  const ev = () => import("@tauri-apps/api/event");
  return {
    modo: "tauri",
    getSnapshot: async () => (await core()).invoke<Snapshot>("get_snapshot"),
    refreshNow: async () => (await core()).invoke<Snapshot>("refresh_now"),
    getHistory: async () => (await core()).invoke<PontoHistorico[]>("get_history"),
    getSettings: async () => (await core()).invoke<Settings>("get_settings"),
    setSettings: async (settings) => void (await core()).invoke("set_settings", { settings }),
    onUpdate: async (cb) => (await ev()).listen<Snapshot>("apuracao:update", (e) => cb(e.payload)),
    onError: async (cb) => (await ev()).listen<string>("apuracao:error", (e) => cb(e.payload)),
  };
}

function mockApi(): Api {
  let passo = 1;
  let settings: Settings = { pollSeconds: 5, mock: true };
  const ups = new Set<(s: Snapshot) => void>();
  let timer: ReturnType<typeof setInterval> | undefined;
  const reagenda = () => {
    if (timer) clearInterval(timer);
    timer = setInterval(() => {
      if (passo < PASSOS_MOCK) {
        passo++;
        const s = mockSnapshotAt(passo);
        ups.forEach((f) => f(s));
      }
    }, Math.max(1, settings.pollSeconds) * 1000);
  };
  return {
    modo: "navegador",
    getSnapshot: async () => mockSnapshotAt(passo),
    refreshNow: async () => {
      passo = Math.min(PASSOS_MOCK, passo + 1);
      return mockSnapshotAt(passo);
    },
    getHistory: async () => mockHistorico(passo),
    getSettings: async () => settings,
    setSettings: async (s) => {
      settings = s;
      reagenda();
    },
    onUpdate: async (cb) => {
      ups.add(cb);
      if (!timer) reagenda();
      return () => {
        ups.delete(cb);
        if (ups.size === 0 && timer) { clearInterval(timer); timer = undefined; }
      };
    },
    onError: async () => () => {},
  };
}

export const api: Api = noTauri() ? tauriApi() : mockApi();
