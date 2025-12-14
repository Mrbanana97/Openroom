import { Eye, EyeOff, Layers, LineChart, Plus, SlidersHorizontal, Trash2 } from "lucide-react";
import { useEffect } from "react";
import { usePreviewState } from "../features/editor/previewState";
import { useRecipeStore } from "../features/editor/recipeStore";
import type { AdjustmentLayer } from "../features/library/types";
import { cn } from "../lib/utils";
import { Button } from "./ui/button";
import { Slider } from "./ui/slider";

const clamp01 = (value: number) => Math.min(1, Math.max(0, value));

function LayerListItem({
  layer,
  active,
  onSelect,
  onToggle,
  onRemove,
}: {
  layer: AdjustmentLayer;
  active: boolean;
  onSelect: () => void;
  onToggle: () => void;
  onRemove: () => void;
}) {
  return (
    <div
      className={cn(
        "flex items-center justify-between gap-2 rounded-lg border px-3 py-2",
        active
          ? "border-[var(--accent)] bg-[var(--accent-soft)] shadow-[0_10px_28px_rgba(0,0,0,0.06)]"
          : "border-[var(--border)] bg-[var(--surface-muted)]/70 hover:border-[var(--border-strong)]",
      )}
    >
      <button
        type="button"
        onClick={onSelect}
        className="flex flex-1 flex-col text-left"
      >
        <div className="flex items-center gap-2 text-sm font-semibold text-[var(--text-primary)]">
          <span className="inline-flex h-2.5 w-2.5 rounded-full bg-[var(--accent)]" />
          {layer.name || "Gradient"}
        </div>
        <div className="text-[11px] text-[var(--text-muted)]">
          {layer.enabled ? "Enabled" : "Disabled"} | {(layer.opacity * 100).toFixed(0)}% opacity
        </div>
      </button>
      <div className="flex items-center gap-1">
        <Button
          size="sm"
          variant="ghost"
          onClick={onToggle}
          title={layer.enabled ? "Hide layer" : "Show layer"}
        >
          {layer.enabled ? <Eye className="h-4 w-4" /> : <EyeOff className="h-4 w-4" />}
        </Button>
        <Button size="sm" variant="ghost" onClick={onRemove} title="Remove layer">
          <Trash2 className="h-4 w-4 text-[var(--text-muted)]" />
        </Button>
      </div>
    </div>
  );
}

export function AdjustmentPanel() {
  const globals = useRecipeStore((state) => state.recipe.globals);
  const layers = useRecipeStore((state) => state.recipe.layers);
  const selectedLayerId = useRecipeStore((state) => state.selectedLayerId);
  const updateGlobals = useRecipeStore((state) => state.updateGlobals);
  const addLayer = useRecipeStore((state) => state.addLayer);
  const updateLayer = useRecipeStore((state) => state.updateLayer);
  const removeLayer = useRecipeStore((state) => state.removeLayer);
  const selectLayer = useRecipeStore((state) => state.selectLayer);
  const setScrubbing = usePreviewState((state) => state.setScrubbing);

  const beginScrub = () => setScrubbing(true);
  const endScrub = () => setScrubbing(false);

  useEffect(() => {
    if (layers.length === 0) {
      selectLayer(undefined);
      return;
    }
    if (!selectedLayerId || !layers.some((layer) => layer.id === selectedLayerId)) {
      selectLayer(layers[0]?.id);
    }
  }, [layers, selectedLayerId, selectLayer]);

  const activeLayer = layers.find((layer) => layer.id === selectedLayerId);

  const updateActiveLayer = (
    updates: Parameters<typeof updateLayer>[1],
  ) => {
    if (!activeLayer) return;
    updateLayer(activeLayer.id, updates);
  };

  const updateMaskPoint = (key: "start" | "end", index: 0 | 1, value: number) => {
    if (!activeLayer) return;
    const next = [...activeLayer.mask[key]] as [number, number];
    next[index] = clamp01(value);
    updateLayer(activeLayer.id, { mask: { [key]: next } });
  };

  const scrubbableProps = { onScrubStart: beginScrub, onScrubEnd: endScrub };

  return (
    <div className="space-y-4">
      <div className="flex items-center gap-2 text-sm font-semibold text-[var(--text-primary)]">
        <SlidersHorizontal className="h-4 w-4 text-[var(--text-muted)]" />
        Global adjustments
      </div>
      <div className="space-y-2">
        <Slider
          label="Exposure"
          value={globals.exposureEv}
          min={-3}
          max={3}
          step={0.1}
          unit=" EV"
          onChange={(v) => updateGlobals({ exposureEv: v })}
          {...scrubbableProps}
        />
        <Slider
          label="Contrast"
          value={globals.contrast}
          onChange={(v) => updateGlobals({ contrast: v })}
          {...scrubbableProps}
        />
        <Slider
          label="Highlights"
          value={globals.highlights}
          onChange={(v) => updateGlobals({ highlights: v })}
          {...scrubbableProps}
        />
        <Slider
          label="Shadows"
          value={globals.shadows}
          onChange={(v) => updateGlobals({ shadows: v })}
          {...scrubbableProps}
        />
        <Slider
          label="Whites"
          value={globals.whites}
          onChange={(v) => updateGlobals({ whites: v })}
          {...scrubbableProps}
        />
        <Slider
          label="Blacks"
          value={globals.blacks}
          onChange={(v) => updateGlobals({ blacks: v })}
          {...scrubbableProps}
        />
        <Slider
          label="Temp"
          value={globals.temp}
          onChange={(v) => updateGlobals({ temp: v })}
          {...scrubbableProps}
        />
        <Slider
          label="Tint"
          value={globals.tint}
          onChange={(v) => updateGlobals({ tint: v })}
          {...scrubbableProps}
        />
        <Slider
          label="Vibrance"
          value={globals.vibrance}
          onChange={(v) => updateGlobals({ vibrance: v })}
          {...scrubbableProps}
        />
        <Slider
          label="Saturation"
          value={globals.saturation}
          onChange={(v) => updateGlobals({ saturation: v })}
          {...scrubbableProps}
        />
      </div>

      <div className="flex items-center gap-2 text-sm font-semibold text-[var(--text-primary)]">
        <LineChart className="h-4 w-4 text-[var(--text-muted)]" />
        Tone curve
      </div>
      <div className="rounded-lg border border-[var(--border)] bg-[var(--surface-muted)]/60 p-3">
        <div className="h-28 w-full rounded-md bg-gradient-to-br from-[var(--surface)] to-[var(--surface-muted)] shadow-inner" />
        <div className="mt-2 text-xs text-[var(--text-muted)]">
          Curve editor placeholder (RGB / luma)
        </div>
      </div>

      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2 text-sm font-semibold text-[var(--text-primary)]">
          <Layers className="h-4 w-4 text-[var(--text-muted)]" />
          Local layers
        </div>
        <Button size="sm" variant="outline" onClick={() => addLayer()}>
          <Plus className="h-4 w-4" />
          Add gradient
        </Button>
      </div>

      {layers.length === 0 ? (
        <div className="rounded-lg border border-[var(--border)] bg-[var(--surface-muted)]/60 px-3 py-3 text-xs text-[var(--text-secondary)]">
          Add a gradient layer to start light local adjustments (exposure / temp / tint / saturation).
        </div>
      ) : (
        <div className="space-y-2">
          {layers.map((layer) => (
            <LayerListItem
              key={layer.id}
              layer={layer}
              active={layer.id === activeLayer?.id}
              onSelect={() => selectLayer(layer.id)}
              onToggle={() => updateLayer(layer.id, { enabled: !layer.enabled })}
              onRemove={() => removeLayer(layer.id)}
            />
          ))}
        </div>
      )}

      {activeLayer && (
        <div className="space-y-3 rounded-lg border border-[var(--border)] bg-[var(--surface-muted)]/60 p-3">
          <div className="flex items-center justify-between text-sm font-semibold text-[var(--text-primary)]">
            Gradient mask
            <div className="flex items-center gap-2 text-xs text-[var(--text-muted)]">
              <label className="inline-flex items-center gap-1">
                <input
                  type="checkbox"
                  checked={activeLayer.enabled}
                  onChange={(e) => updateActiveLayer({ enabled: e.target.checked })}
                />
                Enabled
              </label>
            </div>
          </div>
          <Slider
            label="Opacity"
            value={activeLayer.opacity * 100}
            min={0}
            max={100}
            step={1}
            unit="%"
            onChange={(v) => updateActiveLayer({ opacity: clamp01(v / 100) })}
            {...scrubbableProps}
          />
          <Slider
            label="Feather"
            value={activeLayer.mask.feather * 100}
            min={0}
            max={100}
            step={1}
            unit="%"
            onChange={(v) => updateActiveLayer({ mask: { feather: clamp01(v / 100) } })}
            {...scrubbableProps}
          />
          <div className="grid grid-cols-2 gap-2">
            <Slider
              label="Start X"
              value={activeLayer.mask.start[0] * 100}
              min={0}
              max={100}
              step={1}
              unit="%"
              onChange={(v) => updateMaskPoint("start", 0, v / 100)}
              {...scrubbableProps}
            />
            <Slider
              label="Start Y"
              value={activeLayer.mask.start[1] * 100}
              min={0}
              max={100}
              step={1}
              unit="%"
              onChange={(v) => updateMaskPoint("start", 1, v / 100)}
              {...scrubbableProps}
            />
            <Slider
              label="End X"
              value={activeLayer.mask.end[0] * 100}
              min={0}
              max={100}
              step={1}
              unit="%"
              onChange={(v) => updateMaskPoint("end", 0, v / 100)}
              {...scrubbableProps}
            />
            <Slider
              label="End Y"
              value={activeLayer.mask.end[1] * 100}
              min={0}
              max={100}
              step={1}
              unit="%"
              onChange={(v) => updateMaskPoint("end", 1, v / 100)}
              {...scrubbableProps}
            />
          </div>
          <label className="inline-flex items-center gap-2 text-xs text-[var(--text-secondary)]">
            <input
              type="checkbox"
              checked={activeLayer.mask.invert}
              onChange={(e) => updateActiveLayer({ mask: { invert: e.target.checked } })}
            />
            Invert mask
          </label>

          <div className="mt-2 flex items-center gap-2 text-sm font-semibold text-[var(--text-primary)]">
            Local adjustments
          </div>
          <div className="space-y-2">
            <Slider
              label="Local Exposure"
              value={activeLayer.adjustments.exposureEv}
              min={-2}
              max={2}
              step={0.1}
              unit=" EV"
              onChange={(v) => updateActiveLayer({ adjustments: { exposureEv: v } })}
              {...scrubbableProps}
            />
            <Slider
              label="Local Temp"
              value={activeLayer.adjustments.temp}
              min={-50}
              max={50}
              step={1}
              onChange={(v) => updateActiveLayer({ adjustments: { temp: v } })}
              {...scrubbableProps}
            />
            <Slider
              label="Local Tint"
              value={activeLayer.adjustments.tint}
              min={-50}
              max={50}
              step={1}
              onChange={(v) => updateActiveLayer({ adjustments: { tint: v } })}
              {...scrubbableProps}
            />
            <Slider
              label="Local Saturation"
              value={activeLayer.adjustments.saturation}
              min={-100}
              max={100}
              step={2}
              onChange={(v) => updateActiveLayer({ adjustments: { saturation: v } })}
              {...scrubbableProps}
            />
          </div>
        </div>
      )}
    </div>
  );
}
