import * as SeparatorPrimitive from "@radix-ui/react-separator";
import { cn } from "../../lib/utils";
import type { ComponentPropsWithoutRef } from "react";

type SeparatorProps = ComponentPropsWithoutRef<typeof SeparatorPrimitive.Root>;

export function Separator({ className, decorative = true, ...props }: SeparatorProps) {
  return (
    <SeparatorPrimitive.Root
      decorative={decorative}
      className={cn("h-px bg-[var(--border)]", className)}
      {...props}
    />
  );
}
