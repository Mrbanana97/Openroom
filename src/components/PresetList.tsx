import { Sparkles } from "lucide-react";
import { useState } from "react";
import { useRecipeStore } from "../features/editor/recipeStore";
import type { GlobalAdjustments } from "../features/library/types";
import { Button } from "./ui/button";
import { Slider } from "./ui/slider";

type Preset = {
  name: string;
  mood: string;
  notes: string;
  globals: GlobalAdjustments;
};

const PRESETS: Preset[] = [
  {
    name: "Clean Contrast",
    mood: "Neutral",
    notes: "Crisp whites, gentle black lift, subtle clarity.",
    globals: { exposureEv: 0, contrast: 8, highlights: -6, shadows: 10, whites: 6, blacks: -8, temp: 0, tint: 0, vibrance: 10, saturation: 4 },
  },
  {
    name: "Warm Film",
    mood: "Filmic",
    notes: "Amber warmth with a soft roll-off in highlights.",
    globals: { exposureEv: 0.1, contrast: -4, highlights: -8, shadows: 6, whites: 4, blacks: -6, temp: 12, tint: 2, vibrance: 8, saturation: 6 },
  },
  {
    name: "Cool Fade",
    mood: "Chill",
    notes: "Blue lift in shadows with a matte curve.",
    globals: { exposureEv: -0.05, contrast: -6, highlights: -4, shadows: 12, whites: -2, blacks: 8, temp: -10, tint: 0, vibrance: 6, saturation: -4 },
  },
  {
    name: "B&W Matte",
    mood: "Monochrome",
    notes: "Soft contrast with lifted blacks for portrait-friendly BW.",
    globals: { exposureEv: 0, contrast: -2, highlights: -6, shadows: 8, whites: -4, blacks: 14, temp: 0, tint: 0, vibrance: -100, saturation: -100 },
  },
  {
    name: "Golden Hour",
    mood: "Glow",
    notes: "Warm highlights with gentle saturation for sunsets.",
    globals: { exposureEv: 0.15, contrast: 4, highlights: -6, shadows: 6, whites: 8, blacks: -4, temp: 18, tint: 4, vibrance: 12, saturation: 10 },
  },
];

export function PresetList() {
  const applyPreset = useRecipeStore((state) => state.applyPreset);
  const [intensity, setIntensity] = useState(1);

  return (
    <div className="space-y-3">
      <div className="rounded-lg border border-[var(--border)] bg-[var(--surface-muted)]/60 px-3 py-2">
        <div className="flex items-center justify-between text-sm font-semibold text-[var(--text-primary)]">
          Intensity
          <span className="text-xs text-[var(--text-muted)]">{(intensity * 100).toFixed(0)}%</span>
        </div>
        <div className="mt-2">
          <Slider
            label=""
            value={intensity * 100}
            min={0}
            max={200}
            step={5}
            unit="%"
            onChange={(v) => setIntensity(v / 100)}
          />
        </div>
      </div>
      {PRESETS.map((preset) => (
        <div
          key={preset.name}
          className="rounded-lg border border-[var(--border)] bg-[var(--surface-muted)]/70 px-3 py-3 shadow-[0_6px_22px_rgba(0,0,0,0.03)]"
        >
          <div className="flex items-start justify-between gap-2">
            <div>
              <div className="flex items-center gap-2 text-sm font-semibold text-[var(--text-primary)]">
                <Sparkles className="h-4 w-4 text-[var(--text-muted)]" />
                {preset.name}
              </div>
              <div className="text-[11px] uppercase tracking-wide text-[var(--text-muted)]">
                {preset.mood}
              </div>
            </div>
            <Button
              size="sm"
              variant="ghost"
              onClick={() => applyPreset(preset.globals, intensity)}
              title="Apply preset"
            >
              Apply
            </Button>
          </div>
          <p className="mt-2 text-xs leading-relaxed text-[var(--text-secondary)]">
            {preset.notes}
          </p>
        </div>
      ))}
    </div>
  );
}
