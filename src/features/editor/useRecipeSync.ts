import { invoke } from "@tauri-apps/api/core";
import { useEffect, useRef } from "react";
import { useRecipeStore } from "./recipeStore";
import type { EditRecipe } from "../library/types";
import { createDefaultRecipe } from "../library/types";

export function useRecipeSync(assetId?: string) {
  const recipe = useRecipeStore((state) => state.recipe);
  const setRecipe = useRecipeStore((state) => state.setRecipe);
  const reset = useRecipeStore((state) => state.reset);
  const saveTimer = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);
  const initialLoad = useRef(true);

  // load when asset changes
  useEffect(() => {
    if (!assetId) {
      reset();
      return;
    }
    let cancelled = false;
    async function load() {
      try {
        const loaded = await invoke<EditRecipe | null>("load_recipe", { assetId });
        if (cancelled) return;
        setRecipe(loaded ?? createDefaultRecipe());
        initialLoad.current = true;
      } catch (_err) {
        if (cancelled) return;
        reset();
        initialLoad.current = true;
      }
    }
    void load();
    return () => {
      cancelled = true;
    };
  }, [assetId, reset, setRecipe]);

  // save on change with debounce
  useEffect(() => {
    if (!assetId) return;
    if (saveTimer.current) clearTimeout(saveTimer.current);
    saveTimer.current = setTimeout(() => {
      if (initialLoad.current) {
        initialLoad.current = false;
        return;
      }
      void invoke("save_recipe", { assetId, recipe });
    }, 300);
    return () => {
      if (saveTimer.current) clearTimeout(saveTimer.current);
    };
  }, [assetId, recipe]);
}
