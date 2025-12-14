import { create } from "zustand";
import type { AssetSummary, FolderIndex } from "./types";

type LibraryState = {
  folder?: FolderIndex;
  selectedAssetId?: string;
  loading: boolean;
  preloadActive: boolean;
  preloadDone: number;
  preloadTotal: number;
  setFolder: (folder: FolderIndex) => void;
  setLoading: (loading: boolean) => void;
  setPreloadProgress: (done: number, total: number, active: boolean) => void;
  selectAsset: (id: string) => void;
};

export const useLibraryStore = create<LibraryState>((set) => ({
  folder: undefined,
  selectedAssetId: undefined,
  loading: false,
  preloadActive: false,
  preloadDone: 0,
  preloadTotal: 0,
  setFolder: (folder) =>
    set({
      folder,
      selectedAssetId: folder.assets[0]?.id,
    }),
  setLoading: (loading) => set({ loading }),
  setPreloadProgress: (done, total, active) =>
    set({ preloadDone: done, preloadTotal: total, preloadActive: active }),
  selectAsset: (id) => set({ selectedAssetId: id }),
}));

export const useSelectedAsset = (): AssetSummary | undefined => {
  const folder = useLibraryStore((state) => state.folder);
  const selectedAssetId = useLibraryStore((state) => state.selectedAssetId);
  return folder?.assets.find((asset) => asset.id === selectedAssetId);
};
