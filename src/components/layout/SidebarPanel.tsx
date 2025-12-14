import { Separator } from "../primitives/Separator";
import { cn } from "../../lib/utils";
import type { ReactNode } from "react";

interface SidebarPanelProps {
  title: string;
  children: ReactNode;
  width?: number;
  footer?: ReactNode;
  className?: string;
}

export function SidebarPanel({ title, children, width = 260, footer, className }: SidebarPanelProps) {
  return (
    <div
      className={cn(
        "flex h-full min-w-0 flex-col rounded-xl border border-[var(--border)] bg-[var(--surface)] shadow-sm",
        className,
      )}
      style={{ width }}
    >
      <div className="flex items-center justify-between px-3.5 py-3">
        <div className="text-sm font-semibold text-[var(--text-primary)]">{title}</div>
      </div>
      <Separator />
      <div className="relative h-full flex-1 overflow-y-auto">
        <div className="space-y-3 px-3.5 py-3">{children}</div>
      </div>
      {footer && (
        <>
          <Separator />
          <div className="px-3.5 py-3 text-xs text-[var(--text-muted)]">{footer}</div>
        </>
      )}
    </div>
  );
}
