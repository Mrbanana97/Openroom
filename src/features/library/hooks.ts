import { invoke } from "@tauri-apps/api/core";
import { useEffect, useMemo, useState } from "react";
import type { EditRecipe, Metadata } from "./types";

const PNG_TYPE = "image/png";

function bytesToObjectUrl(bytes: number[]): string {
  const uint8 = new Uint8Array(bytes);
  const blob = new Blob([uint8], { type: PNG_TYPE });
  return URL.createObjectURL(blob);
}

type RenderOptions = {
  maxDimension?: number;
  debounceMs?: number;
  progressive?: boolean;
  progressiveFloor?: number;
  skipHigh?: boolean;
};

function useImageCommand(
  command: string,
  assetId?: string,
  recipe?: EditRecipe,
  options: RenderOptions = {},
) {
  const { maxDimension, debounceMs = 0 } = options;
  const [url, setUrl] = useState<string>();
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string>();

  const payload = useMemo(() => {
    if (!assetId) return null;
    const base = recipe ? { assetId, recipe } : { assetId };
    return maxDimension ? { ...base, maxDimension } : base;
  }, [assetId, recipe, maxDimension]);

  useEffect(() => {
    let active = true;
    const objectUrls: string[] = [];
    let timer: ReturnType<typeof setTimeout> | undefined;

    const revokeAll = () => {
      objectUrls.forEach((u) => URL.revokeObjectURL(u));
    };

    async function load() {
      if (!assetId || !payload) {
        setUrl(undefined);
        setError(undefined);
        return;
      }
      setLoading(true);
      setError(undefined);

      const progressive = options.progressive ?? false;
      const floor = options.progressiveFloor ?? 720;
      const targetDim = maxDimension ?? 1440;
      let fastDim = targetDim;
      if (progressive) {
        fastDim = Math.min(floor, targetDim);
      }
      const doTwoStep = progressive && targetDim > fastDim;

      const fetchDim = async (dim: number) => {
        const bytes = await invoke<number[]>(command, { ...payload, maxDimension: dim });
        if (!active) return null;
        const u = bytesToObjectUrl(bytes);
        objectUrls.push(u);
        return u;
      };

      try {
        if (options.skipHigh) {
          const low = await fetchDim(fastDim);
          if (!active) return;
          if (low) setUrl(low);
        } else if (doTwoStep) {
          const lowPromise = fetchDim(fastDim);
          const highPromise = fetchDim(targetDim);
          const low = await lowPromise;
          if (!active) return;
          if (low) setUrl(low);
          const high = await highPromise;
          if (!active) return;
          if (high) setUrl(high);
        } else {
          const single = await fetchDim(targetDim);
          if (!active) return;
          if (single) setUrl(single);
        }
      } catch (err) {
        if (!active) return;
        setError(err instanceof Error ? err.message : String(err));
      } finally {
        if (active) setLoading(false);
      }
    }

    const run = () => void load();
    if (debounceMs > 0) {
      timer = setTimeout(run, debounceMs);
    } else {
      run();
    }

    return () => {
      active = false;
      revokeAll();
      if (timer) clearTimeout(timer);
    };
  }, [assetId, command, payload, debounceMs, options.progressive, options.progressiveFloor, options.skipHigh]);

  return { url, loading, error };
}

export function useThumbnail(assetId?: string) {
  return useImageCommand("get_thumbnail", assetId);
}

export function usePreview(assetId?: string, recipe?: EditRecipe, opts: RenderOptions = {}) {
  const { maxDimension, debounceMs = 120 } = opts;
  return useImageCommand("render_preview", assetId, recipe, {
    maxDimension,
    debounceMs,
    progressive: opts.progressive ?? false,
    progressiveFloor: opts.progressiveFloor,
  });
}

export function useMetadata(assetId?: string) {
  const [data, setData] = useState<Metadata | undefined>();
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string>();

  useEffect(() => {
    let active = true;
    async function load() {
      if (!assetId) {
        setData(undefined);
        setError(undefined);
        return;
      }
      setLoading(true);
      setError(undefined);
      try {
        const result = await invoke<Metadata>("read_metadata", { assetId });
        if (!active) return;
        setData(result);
      } catch (err) {
        if (!active) return;
        setError(err instanceof Error ? err.message : String(err));
      } finally {
        if (active) setLoading(false);
      }
    }
    void load();
    return () => {
      active = false;
    };
  }, [assetId]);

  return { data, loading, error };
}
