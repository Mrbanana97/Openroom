export type AssetSummary = {
  id: string;
  fileName: string;
  extension: string;
  path: string;
};

export type FolderIndex = {
  id: string;
  path: string;
  assets: AssetSummary[];
};

export type Metadata = {
  camera?: string;
  lens?: string;
  iso?: string;
  shutter?: string;
  aperture?: string;
  focal?: string;
  date?: string;
};

export type GlobalAdjustments = {
  exposureEv: number;
  contrast: number;
  highlights: number;
  shadows: number;
  whites: number;
  blacks: number;
  temp: number;
  tint: number;
  vibrance: number;
  saturation: number;
};

export type Mask = {
  maskType: "linear_gradient";
  start: [number, number];
  end: [number, number];
  feather: number;
  invert: boolean;
};

export type LocalAdjustments = {
  exposureEv: number;
  temp: number;
  tint: number;
  saturation: number;
};

export type AdjustmentLayer = {
  id: string;
  name: string;
  enabled: boolean;
  opacity: number;
  mask: Mask;
  adjustments: LocalAdjustments;
};

export type EditRecipe = {
  version: number;
  globals: GlobalAdjustments;
  layers: AdjustmentLayer[];
};

export type GpuAdapter = {
  name: string;
  backend: string;
  deviceType: string;
};

export const defaultGlobals: GlobalAdjustments = {
  exposureEv: 0,
  contrast: 0,
  highlights: 0,
  shadows: 0,
  whites: 0,
  blacks: 0,
  temp: 0,
  tint: 0,
  vibrance: 0,
  saturation: 0,
};

const uid = () => {
  if (typeof crypto !== "undefined" && "randomUUID" in crypto) {
    return crypto.randomUUID();
  }
  return `layer-${Math.random().toString(16).slice(2)}`;
};

export const createDefaultLayer = (name?: string): AdjustmentLayer => ({
  id: uid(),
  name: name ?? "Gradient",
  enabled: true,
  opacity: 1,
  mask: {
    maskType: "linear_gradient",
    start: [0.35, 0.2],
    end: [0.65, 0.8],
    feather: 0.35,
    invert: false,
  },
  adjustments: {
    exposureEv: 0,
    temp: 0,
    tint: 0,
    saturation: 0,
  },
});

export const createDefaultRecipe = (): EditRecipe => ({
  version: 1,
  globals: { ...defaultGlobals },
  layers: [],
});

export const defaultRecipe: EditRecipe = createDefaultRecipe();
