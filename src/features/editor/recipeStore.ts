import { create } from "zustand";
import type {
  AdjustmentLayer,
  EditRecipe,
  GlobalAdjustments,
  LocalAdjustments,
  Mask,
} from "../library/types";
import { createDefaultLayer, createDefaultRecipe } from "../library/types";

type RecipeState = {
  recipe: EditRecipe;
  selectedLayerId?: string;
  reset: () => void;
  updateGlobals: (partial: Partial<GlobalAdjustments>) => void;
  setRecipe: (recipe: EditRecipe) => void;
  applyPreset: (preset: GlobalAdjustments, intensity: number) => void;
  addLayer: () => void;
  updateLayer: (
    id: string,
    updates: Partial<Omit<AdjustmentLayer, "mask" | "adjustments">> & {
      mask?: Partial<Mask>;
      adjustments?: Partial<LocalAdjustments>;
    },
  ) => void;
  removeLayer: (id: string) => void;
  selectLayer: (id?: string) => void;
};

export const useRecipeStore = create<RecipeState>((set) => ({
  recipe: createDefaultRecipe(),
  selectedLayerId: undefined,
  reset: () => set({ recipe: createDefaultRecipe(), selectedLayerId: undefined }),
  updateGlobals: (partial) =>
    set((state) => ({
      recipe: {
        ...state.recipe,
        globals: { ...state.recipe.globals, ...partial },
      },
    })),
  setRecipe: (recipe) =>
    set({
      recipe,
      selectedLayerId: recipe.layers[0]?.id,
    }),
  applyPreset: (preset, intensity) =>
    set((state) => {
      const t = intensity;
      const base = state.recipe.globals;
      const mix = (a: number, b: number) => a + (b - a) * t;
      return {
        recipe: {
          ...state.recipe,
          globals: {
            exposureEv: mix(base.exposureEv, preset.exposureEv),
            contrast: mix(base.contrast, preset.contrast),
            highlights: mix(base.highlights, preset.highlights),
            shadows: mix(base.shadows, preset.shadows),
            whites: mix(base.whites, preset.whites),
            blacks: mix(base.blacks, preset.blacks),
            temp: mix(base.temp, preset.temp),
            tint: mix(base.tint, preset.tint),
            vibrance: mix(base.vibrance, preset.vibrance),
            saturation: mix(base.saturation, preset.saturation),
          },
        },
      };
    }),
  addLayer: () =>
    set((state) => {
      const layer = createDefaultLayer(`Gradient ${state.recipe.layers.length + 1}`);
      return {
        recipe: { ...state.recipe, layers: [...state.recipe.layers, layer] },
        selectedLayerId: layer.id,
      };
    }),
  updateLayer: (id, updates) =>
    set((state) => {
      const layers = state.recipe.layers.map((layer) => {
        if (layer.id !== id) return layer;
        return {
          ...layer,
          ...updates,
          mask: updates.mask ? { ...layer.mask, ...updates.mask } : layer.mask,
          adjustments: updates.adjustments
            ? { ...layer.adjustments, ...updates.adjustments }
            : layer.adjustments,
        };
      });
      return { recipe: { ...state.recipe, layers } };
    }),
  removeLayer: (id) =>
    set((state) => {
      const layers = state.recipe.layers.filter((layer) => layer.id !== id);
      const selectedLayerId =
        state.selectedLayerId === id ? layers[0]?.id : state.selectedLayerId;
      return { recipe: { ...state.recipe, layers }, selectedLayerId };
    }),
  selectLayer: (id) => set({ selectedLayerId: id }),
}));
