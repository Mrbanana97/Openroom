import { create } from "zustand";

type PreviewState = {
  isScrubbing: boolean;
  setScrubbing: (value: boolean) => void;
};

export const usePreviewState = create<PreviewState>((set) => ({
  isScrubbing: false,
  setScrubbing: (value) => set({ isScrubbing: value }),
}));
