import { Aperture, CalendarDays, Camera, Ruler, Timer, Waves } from "lucide-react";
import { useMetadata } from "../features/library/hooks";
import { useSelectedAsset } from "../features/library/store";

function MetaItem({
  icon,
  label,
  value,
}: {
  icon: React.ReactNode;
  label: string;
  value?: string;
}) {
  return (
    <div className="flex items-center gap-2 rounded-lg border border-[var(--border)] bg-[var(--surface-muted)]/70 px-3 py-2">
      <span className="text-[var(--text-muted)]">{icon}</span>
      <div className="flex flex-col leading-tight">
        <span className="text-[11px] uppercase tracking-wide text-[var(--text-muted)]">
          {label}
        </span>
        <span className="text-sm text-[var(--text-primary)] min-h-[18px]">
          {value ?? "-"}
        </span>
      </div>
    </div>
  );
}

export function MetadataBar() {
  const asset = useSelectedAsset();
  const { data, loading, error } = useMetadata(asset?.id);

  if (!asset) return null;

  return (
    <div className="w-full rounded-xl border border-[var(--border)] bg-[var(--surface)] px-4 py-3 shadow-sm">
      <div className="flex items-center justify-between gap-3">
        <div className="text-sm font-semibold text-[var(--text-primary)]">
          Metadata
          <span className="ml-2 text-xs text-[var(--text-muted)]">
            {loading ? "Reading EXIF..." : error ? "EXIF unavailable" : "EXIF read"}
          </span>
        </div>
        <div className="text-xs text-[var(--text-muted)]">{asset.path}</div>
      </div>
      <div className="mt-3 grid grid-cols-2 gap-2 sm:grid-cols-3 md:grid-cols-6">
        <MetaItem icon={<Camera className="h-4 w-4" />} label="Camera" value={data?.camera} />
        <MetaItem icon={<Waves className="h-4 w-4" />} label="Lens" value={data?.lens} />
        <MetaItem icon={<Aperture className="h-4 w-4" />} label="Aperture" value={data?.aperture} />
        <MetaItem icon={<Timer className="h-4 w-4" />} label="Shutter" value={data?.shutter} />
        <MetaItem icon={<Ruler className="h-4 w-4" />} label="Focal" value={data?.focal} />
        <MetaItem icon={<CalendarDays className="h-4 w-4" />} label="Date" value={data?.date} />
      </div>
      {error && (
        <div className="mt-2 text-[11px] text-[var(--text-muted)]">
          {error}
        </div>
      )}
    </div>
  );
}
