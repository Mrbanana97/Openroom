import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { useLibraryStore } from "./store";
import type { FolderIndex } from "./types";

async function preloadThumbnails(assets: FolderIndex["assets"]) {
  const setProgress = useLibraryStore.getState().setPreloadProgress;
  const total = assets.length;
  if (total === 0) {
    setProgress(0, 0, false);
    return;
  }

  let done = 0;
  const concurrency = 2;
  const queue = assets.slice();

  const worker = async () => {
    while (queue.length > 0) {
      const asset = queue.shift();
      if (!asset) break;
      try {
        await invoke<number[]>("get_thumbnail", { assetId: asset.id });
      } catch (err) {
        console.warn("Failed to preload thumbnail", asset.fileName, err);
      } finally {
        done += 1;
        setProgress(done, total, true);
      }
    }
  };

  setProgress(0, total, true);
  await Promise.all(Array.from({ length: concurrency }, () => worker()));
  setProgress(total, total, false);
}

export async function pickFolderAndLoad(): Promise<FolderIndex | null> {
  const setLoading = useLibraryStore.getState().setLoading;
  try {
    setLoading(true);
    const selection = await open({
      directory: true,
      multiple: false,
      recursive: false,
      title: "Select a photo folder",
    });

    const folderPath = Array.isArray(selection) ? selection[0] : selection;
    if (!folderPath) {
      setLoading(false);
      return null;
    }

    const folderIndex = await invoke<FolderIndex>("open_folder", { path: folderPath });
    useLibraryStore.getState().setFolder(folderIndex);
    void preloadThumbnails(folderIndex.assets);
    setLoading(false);
    return folderIndex;
  } catch (error) {
    setLoading(false);
    console.error("Failed to open folder", error);
    return null;
  }
}

export async function loadFolderFromPath(path: string): Promise<FolderIndex | null> {
  const setLoading = useLibraryStore.getState().setLoading;
  try {
    setLoading(true);
    const folderIndex = await invoke<FolderIndex>("open_folder", { path });
    useLibraryStore.getState().setFolder(folderIndex);
    void preloadThumbnails(folderIndex.assets);
    setLoading(false);
    return folderIndex;
  } catch (error) {
    setLoading(false);
    console.error("Failed to load folder", error);
    return null;
  }
}
