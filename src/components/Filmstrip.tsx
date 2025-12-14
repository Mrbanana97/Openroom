import { Image, Loader2 } from "lucide-react";
import { useThumbnail } from "../features/library/hooks";
import { useLibraryStore } from "../features/library/store";
import type { AssetSummary } from "../features/library/types";
import { cn } from "../lib/utils";

function FilmstripItem({
  asset,
  active,
  onSelect,
}: {
  asset: AssetSummary;
  active: boolean;
  onSelect: () => void;
}) {
  const { url, loading } = useThumbnail(asset.id);

  return (
    <button
      type="button"
      onClick={onSelect}
      className={cn(
        "group relative flex h-20 min-w-[120px] flex-col rounded-lg border px-3 py-2 text-left transition-colors",
        active
          ? "border-[var(--accent)] bg-[var(--accent-soft)] shadow-[0_12px_34px_rgba(0,0,0,0.06)]"
          : "border-[var(--border)] bg-[var(--surface)] hover:border-[var(--border-strong)] hover:bg-[var(--surface-muted)]",
      )}
    >
      <div className="flex flex-1 items-center gap-2 text-[var(--text-secondary)]">
        <span className="flex h-12 w-16 items-center justify-center overflow-hidden rounded-md border border-[var(--border)] bg-[var(--surface-muted)] text-[var(--text-muted)]">
          {url ? (
            <img src={url} alt={asset.fileName} className="h-full w-full object-cover" />
          ) : loading ? (
            <Loader2 className="h-4 w-4 animate-spin" />
          ) : (
            <Image className="h-5 w-5" />
          )}
        </span>
        <div className="flex-1 truncate">
          <div className="truncate text-sm font-medium text-[var(--text-primary)]">
            {asset.fileName}
          </div>
          <div className="text-[11px] uppercase text-[var(--text-muted)]">
            {asset.extension || "file"}
          </div>
        </div>
      </div>
    </button>
  );
}

export function Filmstrip() {
  const assets = useLibraryStore((state) => state.folder?.assets);
  const selectedAssetId = useLibraryStore((state) => state.selectedAssetId);
  const selectAsset = useLibraryStore((state) => state.selectAsset);
  const loading = useLibraryStore((state) => state.loading);
  const preloadActive = useLibraryStore((state) => state.preloadActive);
  const preloadDone = useLibraryStore((state) => state.preloadDone);
  const preloadTotal = useLibraryStore((state) => state.preloadTotal);
  const list = assets ?? [];
  const showSkeletons = loading && list.length === 0;
  const preloadPercent =
    preloadTotal > 0 ? Math.round((preloadDone / preloadTotal) * 100) : 0;

  return (
    <div className="rounded-2xl border border-[var(--border)] bg-[var(--surface)] shadow-sm">
      <div className="flex items-center justify-between px-4 py-3">
        <div className="flex items-center gap-2 text-sm font-semibold text-[var(--text-primary)]">
          <span>Filmstrip</span>
          {loading && <Loader2 className="h-3.5 w-3.5 animate-spin text-[var(--accent)]" />}
        </div>
        <div className="text-xs text-[var(--text-muted)]">
          {loading
            ? "Loading RAWs..."
            : list.length
              ? `${list.length} photos`
              : "Waiting for a folder"}
        </div>
      </div>
      {(loading || preloadActive) && (
        <div className="mx-4 mb-1 h-1 overflow-hidden rounded-full bg-[var(--surface-muted)]">
          <div
            className="h-full animate-shimmer rounded-full bg-[var(--accent)]"
            style={{ width: preloadTotal > 0 ? `${preloadPercent}%` : "35%" }}
          />
        </div>
      )}
      {preloadActive && (
        <div className="px-4 pb-1 text-[11px] text-[var(--text-muted)]">
          Prefetching thumbnails {preloadDone}/{preloadTotal} ({preloadPercent}%)
        </div>
      )}
      <div className="w-full overflow-x-auto">
        <div className="flex min-h-[96px] items-center gap-3 px-4 pb-3">
          {showSkeletons ? (
            Array.from({ length: 6 }).map((_, idx) => (
              <div
                key={`skeleton-${idx}`}
                className="flex h-20 min-w-[120px] flex-col rounded-lg border border-[var(--border)] bg-[var(--surface-muted)] p-3 shadow-inner"
              >
                <div className="flex gap-2">
                  <div className="h-12 w-16 animate-pulse rounded-md bg-[var(--border)]" />
                  <div className="flex flex-1 flex-col gap-2">
                    <div className="h-3 w-20 animate-pulse rounded bg-[var(--border)]" />
                    <div className="h-2.5 w-14 animate-pulse rounded bg-[var(--border)]" />
                  </div>
                </div>
              </div>
            ))
          ) : list.length === 0 ? (
            <div className="flex items-center gap-2 text-sm text-[var(--text-muted)]">
              <Loader2 className="h-4 w-4 animate-spin" />
              No assets yet. Open a folder to see thumbnails.
            </div>
          ) : (
            list.map((asset) => (
              <FilmstripItem
                key={asset.id}
                asset={asset}
                active={asset.id === selectedAssetId}
                onSelect={() => selectAsset(asset.id)}
              />
            ))
          )}
        </div>
      </div>
    </div>
  );
}
