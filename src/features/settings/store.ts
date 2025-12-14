import { invoke } from "@tauri-apps/api/core";
import { create } from "zustand";
import type { GpuAdapter } from "../library/types";

type SettingsState = {
  gpuAcceleration: boolean;
  adapters: GpuAdapter[];
  selectedAdapterName?: string;
  lastError?: string;
  setGpuAcceleration: (enabled: boolean) => void;
  detectAdapters: () => Promise<void>;
  selectAdapter: (name?: string) => void;
};

export const useSettingsStore = create<SettingsState>((set) => ({
  gpuAcceleration: false,
  adapters: [],
  selectedAdapterName: undefined,
  lastError: undefined,
  setGpuAcceleration: (enabled) => set({ gpuAcceleration: enabled }),
  selectAdapter: (name) => set({ selectedAdapterName: name }),
  detectAdapters: async () => {
    try {
      const adapters = await invoke<GpuAdapter[]>("detect_gpus");
      set({
        adapters,
        selectedAdapterName: adapters[0]?.name,
        lastError: undefined,
      });
    } catch (error) {
      set({
        adapters: [],
        lastError: error instanceof Error ? error.message : String(error),
      });
    }
  },
}));
