import { cn } from "../../lib/utils";

type SliderProps = {
  label: string;
  value: number;
  min?: number;
  max?: number;
  step?: number;
  unit?: string;
  onChange: (value: number) => void;
  onScrubStart?: () => void;
  onScrubEnd?: () => void;
};

export function Slider({
  label,
  value,
  min = -100,
  max = 100,
  step = 1,
  unit,
  onChange,
  onScrubStart,
  onScrubEnd,
}: SliderProps) {
  return (
    <div className="flex flex-col gap-1 rounded-lg border border-[var(--border)] bg-[var(--surface-muted)]/60 px-3 py-2">
      <div className="flex items-center justify-between text-xs">
        <span className="font-semibold uppercase tracking-wide text-[var(--text-muted)]">{label}</span>
        <span className="text-[var(--text-secondary)]">
          {value.toFixed(1)}
          {unit ? unit : ""}
        </span>
      </div>
      <input
        type="range"
        min={min}
        max={max}
        step={step}
        value={value}
        onChange={(e) => onChange(parseFloat(e.target.value))}
        onPointerDown={onScrubStart}
        onPointerUp={onScrubEnd}
        className={cn(
          "h-1.5 w-full appearance-none rounded-full bg-[var(--border)]",
          "accent-[var(--accent)] outline-none",
        )}
      />
    </div>
  );
}
