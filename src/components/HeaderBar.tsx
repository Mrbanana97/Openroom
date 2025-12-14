import { Download, FolderOpen, Redo2, SplitSquareHorizontal, Undo2 } from "lucide-react";
import { pickFolderAndLoad } from "../features/library/actions";
import { useLibraryStore, useSelectedAsset } from "../features/library/store";
import { Button } from "./ui/button";

export function HeaderBar() {
  const folderPath = useLibraryStore((state) => state.folder?.path);
  const selectedAsset = useSelectedAsset();

  return (
    <header className="sticky top-0 z-10 border-b border-[var(--border)] bg-[var(--surface)]/90 backdrop-blur-xl">
      <div className="mx-auto flex max-w-screen-2xl flex-wrap items-center justify-between gap-3 px-4 py-3">
        <div className="flex min-w-0 items-center gap-3">
          <div className="flex items-center gap-2 rounded-lg bg-[var(--surface-muted)] px-3 py-1.5">
            <div className="text-sm font-semibold text-[var(--text-primary)]">Openroom</div>
            <div className="text-[11px] text-[var(--text-muted)]">RAW editor MVP</div>
          </div>
          <div className="truncate text-xs text-[var(--text-muted)]">
            {selectedAsset ? (
              <>
                {selectedAsset.fileName}
                <span className="text-[11px] text-[var(--text-secondary)]">
                  {" "}
                  - {folderPath}
                </span>
              </>
            ) : folderPath ? (
              `Folder: ${folderPath}`
            ) : (
              "Pick a folder to start"
            )}
          </div>
        </div>

        <div className="flex flex-wrap items-center gap-2">
          <Button
            className="min-w-[132px]"
            variant="outline"
            onClick={() => void pickFolderAndLoad()}
          >
            <FolderOpen className="h-4 w-4" />
            Open folder
          </Button>
          <Button variant="soft" size="sm" title="Export (coming soon)" disabled>
            <Download className="h-4 w-4" />
            Export
          </Button>
          <Button
            variant="ghost"
            size="sm"
            title="Before/After (coming soon)"
            disabled
          >
            <SplitSquareHorizontal className="h-4 w-4" />
            Before/After
          </Button>
          <Button variant="ghost" size="sm" title="Undo (soon)" disabled>
            <Undo2 className="h-4 w-4" />
          </Button>
          <Button variant="ghost" size="sm" title="Redo (soon)" disabled>
            <Redo2 className="h-4 w-4" />
          </Button>
        </div>
      </div>
    </header>
  );
}
