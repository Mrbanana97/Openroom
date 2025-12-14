import { Image as ImageIcon, Loader2, ZoomIn, ZoomOut } from "lucide-react";
import { useEffect, useMemo, useRef, useState } from "react";
import { usePreviewState } from "../features/editor/previewState";
import { useRecipeStore } from "../features/editor/recipeStore";
import { usePreview } from "../features/library/hooks";
import { useSelectedAsset } from "../features/library/store";
import type { AdjustmentLayer } from "../features/library/types";
import { MetadataBar } from "./MetadataBar";

type Frame = { width: number; height: number; left: number; top: number };
const clamp01 = (v: number) => Math.min(1, Math.max(0, v));

function GradientOverlay({
  layer,
  frame,
  onDragHandle,
}: {
  layer: AdjustmentLayer;
  frame: Frame;
  onDragHandle: (handle: "start" | "end", clientX: number, clientY: number) => void;
}) {
  const overlayRef = useRef<HTMLDivElement>(null);
  const start = useMemo(
    () => ({
      x: frame.left + layer.mask.start[0] * frame.width,
      y: frame.top + layer.mask.start[1] * frame.height,
    }),
    [frame, layer.mask.start],
  );
  const end = useMemo(
    () => ({
      x: frame.left + layer.mask.end[0] * frame.width,
      y: frame.top + layer.mask.end[1] * frame.height,
    }),
    [frame, layer.mask.end],
  );

  const [dragging, setDragging] = useState<"start" | "end" | null>(null);

  useEffect(() => {
    if (!dragging) return;
    const move = (event: PointerEvent) => onDragHandle(dragging, event.clientX, event.clientY);
    const stop = () => setDragging(null);
    window.addEventListener("pointermove", move);
    window.addEventListener("pointerup", stop);
    return () => {
      window.removeEventListener("pointermove", move);
      window.removeEventListener("pointerup", stop);
    };
  }, [dragging, onDragHandle]);

  return (
    <div ref={overlayRef} className="pointer-events-none absolute inset-0 select-none">
      <svg className="absolute inset-0 h-full w-full" aria-hidden>
        <line
          x1={start.x}
          y1={start.y}
          x2={end.x}
          y2={end.y}
          stroke="rgba(59,130,246,0.9)"
          strokeWidth={2}
          strokeDasharray="6 4"
        />
      </svg>
      {[{ pos: start, type: "start" as const }, { pos: end, type: "end" as const }].map(({ pos, type }) => (
        <button
          key={type}
          type="button"
          className="pointer-events-auto absolute h-4 w-4 -translate-x-1/2 -translate-y-1/2 rounded-full border border-[var(--accent)] bg-white shadow"
          style={{ left: pos.x, top: pos.y }}
          onPointerDown={(e) => {
            e.preventDefault();
            e.stopPropagation();
            setDragging(type);
          }}
          onClick={(e) => e.stopPropagation()}
          aria-label={`Drag gradient ${type} handle`}
        />
      ))}
    </div>
  );
}

export function PreviewStage() {
  const asset = useSelectedAsset();
  const recipe = useRecipeStore((state) => state.recipe);
  const selectedLayerId = useRecipeStore((state) => state.selectedLayerId);
  const updateLayer = useRecipeStore((state) => state.updateLayer);
  const activeLayer = recipe.layers.find((layer) => layer.id === selectedLayerId);
  const isScrubbing = usePreviewState((state) => state.isScrubbing);

  const viewportRef = useRef<HTMLDivElement>(null);
  const [viewportSize, setViewportSize] = useState<{ w: number; h: number }>({ w: 0, h: 0 });
  const [naturalSize, setNaturalSize] = useState<{ w: number; h: number } | null>(null);
  const [frame, setFrame] = useState<Frame | null>(null);
  const [zoomPercent, setZoomPercent] = useState(50);
  const [zoomEnabled, setZoomEnabled] = useState(false);

  const fitSize = useMemo(() => {
    if (!naturalSize || viewportSize.w === 0 || viewportSize.h === 0) return null;
    const aspect = naturalSize.h === 0 ? 1 : naturalSize.w / naturalSize.h;
    let width = viewportSize.w;
    let height = width / aspect;
    if (height > viewportSize.h) {
      height = viewportSize.h;
      width = height * aspect;
    }
    return { width, height };
  }, [naturalSize, viewportSize]);

  const zoomScale = zoomEnabled ? 100 / Math.max(zoomPercent, 1) : 1;

  const maxDimension = useMemo(() => {
    const dpr = typeof window !== "undefined" ? window.devicePixelRatio || 1 : 1;
    const baseWidth = fitSize ? fitSize.width : viewportSize.w;
    const baseHeight = fitSize ? fitSize.height : viewportSize.h;

    if (baseWidth === 0 || baseHeight === 0) return isScrubbing ? 720 : 1280;

    const displayWidth = baseWidth * zoomScale;
    const displayHeight = baseHeight * zoomScale;
    const rawTarget = Math.max(displayWidth, displayHeight) * dpr;
    const renderCap = Math.max(baseWidth, baseHeight) * 5;

    const baseTarget = Math.min(Math.max(rawTarget, 480), renderCap);

    if (isScrubbing) {
      return Math.round(Math.min(baseTarget, Math.max(rawTarget * 0.7, 960)));
    }

    return Math.round(baseTarget);
  }, [fitSize, viewportSize, zoomScale, isScrubbing]);

  const wantsProgressive = zoomEnabled || isScrubbing;
  const progressiveFloor = Math.max(420, Math.round(maxDimension * 0.4));

  const { url, loading, error } = usePreview(asset?.id, recipe, {
    maxDimension,
    debounceMs: isScrubbing ? 8 : 80,
    progressive: wantsProgressive,
    progressiveFloor,
    skipHigh: isScrubbing,
  });

  useEffect(() => {
    const viewportEl = viewportRef.current;
    if (!viewportEl) return;

    const measure = () => {
      const rect = viewportEl.getBoundingClientRect();
      setViewportSize({ w: rect.width, h: rect.height });
    };

    measure();

    const observer = new ResizeObserver(measure);
    observer.observe(viewportEl);
    return () => observer.disconnect();
  }, []);

  useEffect(() => {
    setFrame(null);
    setNaturalSize(null);
    setZoomEnabled(false);
  }, [asset?.id]);

  useEffect(() => {
    if (!fitSize) return;
    const width = fitSize.width * zoomScale;
    const height = fitSize.height * zoomScale;
    setFrame({
      width,
      height,
      left: (viewportSize.w - width) / 2,
      top: (viewportSize.h - height) / 2,
    });
  }, [fitSize, viewportSize, zoomScale]);

  const toNormalized = (clientX: number, clientY: number) => {
    if (!frame || !viewportRef.current) return null;
    const rect = viewportRef.current.getBoundingClientRect();
    const x = clientX - rect.left - frame.left;
    const y = clientY - rect.top - frame.top;
    return [clamp01(x / frame.width), clamp01(y / frame.height)] as [number, number];
  };

  const handleDrag = (handle: "start" | "end", clientX: number, clientY: number) => {
    if (!activeLayer) return;
    const next = toNormalized(clientX, clientY);
    if (!next) return;
    updateLayer(activeLayer.id, { mask: { [handle]: next } });
  };

  return (
    <>
      <div className="flex min-h-[420px] flex-1 overflow-hidden rounded-2xl border border-[var(--border)] bg-[var(--surface)] shadow-sm">
        <div className="relative flex flex-1 items-center justify-center bg-[var(--surface-muted)]">
          <div className="absolute inset-0 bg-gradient-to-br from-[#e8ecf7] via-[var(--surface-muted)] to-[#dfe6ff] opacity-70" />
          <div className="relative z-10 flex w-full flex-col items-center gap-3 px-8 text-center">
            <div
              ref={viewportRef}
              className={`group relative flex h-[60vh] min-h-[420px] w-full max-w-[1400px] items-center justify-center overflow-hidden rounded-xl border border-[var(--border)] bg-white/80 shadow-[0_10px_60px_rgba(0,0,0,0.04)] backdrop-blur-sm ${
                url ? (zoomEnabled ? "cursor-zoom-out" : "cursor-zoom-in") : "cursor-default"
              }`}
              onClick={() => {
                if (!asset) return;
                if (!zoomEnabled) {
                  setZoomPercent(50);
                }
                setZoomEnabled((prev) => !prev);
              }}
            >
              {url ? (
                <>
                  <div
                    className="absolute"
                    style={
                      frame
                        ? {
                            width: frame.width,
                            height: frame.height,
                            left: frame.left,
                            top: frame.top,
                          }
                        : {
                            width: "100%",
                            height: "100%",
                            left: 0,
                            top: 0,
                          }
                    }
                  >
                    <img
                      src={url}
                      alt={asset?.fileName}
                      className="h-full w-full select-none object-contain"
                      style={frame ? undefined : { opacity: 0 }}
                      draggable={false}
                      onLoad={(e) => {
                        setNaturalSize({
                          w: e.currentTarget.naturalWidth,
                          h: e.currentTarget.naturalHeight,
                        });
                      }}
                    />
                  </div>
                  {!frame && (
                    <div className="flex flex-col items-center gap-2 text-[var(--text-secondary)]">
                      <Loader2 className="h-6 w-6 animate-spin" />
                      <div className="text-sm font-medium text-[var(--text-primary)]">
                        Sizing preview...
                      </div>
                    </div>
                  )}
                  {frame && activeLayer && activeLayer.enabled && (
                    <GradientOverlay layer={activeLayer} frame={frame} onDragHandle={handleDrag} />
                  )}
                </>
              ) : asset ? (
                <div className="flex flex-col items-center gap-2 text-[var(--text-secondary)]">
                  {loading ? <Loader2 className="h-6 w-6 animate-spin" /> : <ImageIcon className="h-8 w-8" />}
                  <div className="text-sm font-medium text-[var(--text-primary)]">
                    {loading ? "Rendering preview..." : "Preview not available yet"}
                  </div>
                  {error && (
                    <div className="max-w-sm text-xs text-[var(--text-muted)]">
                      {error}
                    </div>
                  )}
                </div>
              ) : (
                <div className="flex flex-col items-center gap-2 text-[var(--text-muted)]">
                  <ImageIcon className="h-8 w-8" />
                  <div className="text-sm font-medium text-[var(--text-primary)]">
                    Open a folder to generate previews
                  </div>
                  <p className="max-w-md text-xs text-[var(--text-muted)]">
                    We&apos;ll decode RAWs to a fit-to-window preview here. Local tools and global
                    sliders will update this pane in the next steps.
                  </p>
                </div>
              )}
              {asset && (
                <div
                  className="absolute bottom-3 left-3 flex items-center gap-3 rounded-lg border border-[var(--border)] bg-white/85 px-3 py-2 text-xs text-[var(--text-secondary)] shadow-sm backdrop-blur"
                  onClick={(e) => e.stopPropagation()}
                >
                  <div className="flex items-center gap-1 font-semibold text-[var(--text-primary)]">
                    {zoomEnabled ? <ZoomOut className="h-3.5 w-3.5" /> : <ZoomIn className="h-3.5 w-3.5" />}
                    <span>{zoomEnabled ? `${zoomPercent}%` : "Fit"}</span>
                    {zoomEnabled && (
                      <span className="text-[11px] font-normal text-[var(--text-muted)]">
                        · {zoomScale.toFixed(1)}x
                      </span>
                    )}
                  </div>
                  <input
                    type="range"
                    min={25}
                    max={200}
                    step={5}
                    value={zoomPercent}
                    className="pointer-events-auto h-1.5 w-32 accent-[var(--accent)]"
                    onChange={(e) => {
                      const next = Number(e.target.value);
                      setZoomPercent(next);
                      setZoomEnabled(true);
                    }}
                  />
                  <button
                    type="button"
                    className="pointer-events-auto rounded-md border border-[var(--border)] px-2 py-1 text-[11px] font-medium text-[var(--text-primary)] transition hover:border-[var(--border-strong)] hover:bg-[var(--surface-muted)]"
                    onClick={() => setZoomEnabled(false)}
                  >
                    Fit
                  </button>
                </div>
              )}
            </div>
            {asset && (
              <div className="flex flex-col items-center gap-1 rounded-lg bg-white/70 px-4 py-3 text-xs text-[var(--text-secondary)] shadow-[0_12px_40px_rgba(0,0,0,0.06)] backdrop-blur-sm">
                <div className="text-sm font-semibold text-[var(--text-primary)]">
                  {asset.fileName}
                </div>
                <div className="text-[11px] text-[var(--text-muted)]">{asset.path}</div>
                <div className="text-[11px] text-[var(--text-muted)]">
                  Drag gradient handles on the image to shape local adjustments.
                </div>
              </div>
            )}
          </div>
        </div>
      </div>
      <div className="mt-3">
        <MetadataBar />
      </div>
    </>
  );
}
