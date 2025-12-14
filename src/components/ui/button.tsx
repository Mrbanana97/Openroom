import { Slot } from "@radix-ui/react-slot";
import { cva, type VariantProps } from "class-variance-authority";
import * as React from "react";
import { cn } from "../../lib/utils";

const buttonVariants = cva(
  "inline-flex items-center justify-center gap-2 rounded-md text-sm font-medium transition-colors duration-150 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)] focus-visible:ring-offset-2 focus-visible:ring-offset-[var(--surface)] disabled:cursor-not-allowed disabled:opacity-60",
  {
    variants: {
      variant: {
        solid:
          "bg-[var(--accent)] text-white shadow-sm hover:bg-[#1f55c5] active:bg-[#1b4aa9]",
        outline:
          "border border-[var(--border)] bg-[var(--surface)] text-[var(--text-primary)] hover:border-[var(--border-strong)] hover:bg-[var(--surface-muted)]",
        ghost: "bg-transparent text-[var(--text-primary)] hover:bg-[var(--surface-muted)]",
        soft:
          "bg-[var(--accent-soft)] text-[var(--text-primary)] border border-[var(--border)] hover:border-[var(--border-strong)]",
      },
      size: {
        sm: "h-8 px-3 text-xs",
        md: "h-9 px-3.5 text-sm",
        lg: "h-10 px-4 text-sm",
      },
    },
    defaultVariants: {
      variant: "outline",
      size: "md",
    },
  },
);

export interface ButtonProps
  extends React.ButtonHTMLAttributes<HTMLButtonElement>,
    VariantProps<typeof buttonVariants> {
  asChild?: boolean;
}

export const Button = React.forwardRef<HTMLButtonElement, ButtonProps>(
  ({ className, variant, size, asChild, ...props }, ref) => {
    const Comp = asChild ? Slot : "button";
    return (
      <Comp
        ref={ref}
        className={cn(buttonVariants({ variant, size }), className)}
        {...props}
      />
    );
  },
);
Button.displayName = "Button";
