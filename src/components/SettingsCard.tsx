import { useEffect } from "react";
import { useSettingsStore } from "../features/settings/store";
import { cn } from "../lib/utils";

export function SettingsCard({ className }: { className?: string }) {
  const gpuAcceleration = useSettingsStore((state) => state.gpuAcceleration);
  const setGpuAcceleration = useSettingsStore((state) => state.setGpuAcceleration);
  const adapters = useSettingsStore((state) => state.adapters);
  const selectedAdapterName = useSettingsStore((state) => state.selectedAdapterName);
  const selectAdapter = useSettingsStore((state) => state.selectAdapter);
  const detectAdapters = useSettingsStore((state) => state.detectAdapters);
  const lastError = useSettingsStore((state) => state.lastError);

  useEffect(() => {
    if (adapters.length === 0) {
      void detectAdapters();
    }
  }, [adapters.length, detectAdapters]);

  return (
    <div
      className={cn(
        "rounded-xl border border-[var(--border)] bg-[var(--surface-muted)]/60 p-4 text-sm shadow-sm",
        className,
      )}
    >
      <div className="flex items-center justify-between">
        <div>
          <div className="font-semibold text-[var(--text-primary)]">GPU acceleration</div>
          <div className="text-xs text-[var(--text-secondary)]">
            Offloads preview math to your discrete/fast GPU when available.
          </div>
        </div>
        <label className="inline-flex cursor-pointer items-center gap-2 text-xs text-[var(--text-secondary)]">
          <input
            type="checkbox"
            checked={gpuAcceleration}
            onChange={(e) => setGpuAcceleration(e.target.checked)}
          />
          Enable
        </label>
      </div>

      <div className="mt-3 space-y-2">
        <div className="text-xs font-semibold uppercase tracking-wide text-[var(--text-muted)]">
          Detected GPUs
        </div>
        {adapters.length === 0 ? (
          <div className="text-xs text-[var(--text-secondary)]">
            {lastError ? `Detection failed: ${lastError}` : "Scanning adapters..."}
          </div>
        ) : (
          <div className="space-y-1">
            {adapters.map((gpu) => (
              <label
                key={gpu.name + gpu.backend}
                className={cn(
                  "flex cursor-pointer items-center justify-between rounded-lg border px-3 py-2 text-xs",
                  selectedAdapterName === gpu.name
                    ? "border-[var(--accent)] bg-[var(--accent-soft)]"
                    : "border-[var(--border)] bg-[var(--surface)] hover:border-[var(--border-strong)]",
                )}
              >
                <div className="flex flex-col">
                  <span className="font-semibold text-[var(--text-primary)]">{gpu.name}</span>
                  <span className="text-[var(--text-muted)]">
                    {gpu.deviceType} • {gpu.backend}
                  </span>
                </div>
                <input
                  type="radio"
                  name="gpu-selection"
                  checked={selectedAdapterName === gpu.name}
                  onChange={() => selectAdapter(gpu.name)}
                />
              </label>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
