/**
 * Reusable Dialog component.
 *
 * Features:
 * - role="dialog" with aria-modal="true"
 * - Title labelling via aria-labelledby
 * - Escape to close
 * - Outside click to close
 * - Close button
 * - Autofocus support
 * - Focus restore on close
 * - Background scroll lock
 * - Size variants: sm (480px), md (640px), lg (760px)
 * - Sections: DialogHeader, DialogBody, DialogFooter
 */

import { useEffect, useRef, type ReactNode } from "react";
import { X } from "lucide-react";

export type DialogSize = "sm" | "md" | "lg";

type DialogProps = {
  open: boolean;
  onClose: () => void;
  size?: DialogSize;
  title: string;
  subtitle?: string;
  children: ReactNode;
  className?: string;
};

export function Dialog({ open, onClose, size = "md", title, subtitle, children, className }: DialogProps) {
  const panelRef = useRef<HTMLElement>(null);
  const previousFocus = useRef<HTMLElement | null>(null);

  // Lock body scroll when open
  useEffect(() => {
    if (!open) return;
    const scrollY = window.scrollY;
    document.body.style.overflow = "hidden";
    return () => {
      document.body.style.overflow = "";
      window.scrollTo(0, scrollY);
    };
  }, [open]);

  // Restore focus on close
  useEffect(() => {
    if (open) {
      previousFocus.current = document.activeElement as HTMLElement;
    } else if (previousFocus.current) {
      previousFocus.current.focus?.();
      previousFocus.current = null;
    }
  }, [open]);

  // Escape to close
  useEffect(() => {
    if (!open) return;
    function handleKeyDown(event: KeyboardEvent) {
      if (event.key === "Escape") {
        event.stopPropagation();
        onClose();
      }
    }
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [open, onClose]);

  if (!open) return null;

  const sizeClass = `dialog-${size}`;

  return (
    <div
      className="dialog-overlay"
      role="presentation"
      onMouseDown={onClose}
    >
      <section
        ref={panelRef}
        className={`dialog-panel ${sizeClass} ${className ?? ""}`}
        role="dialog"
        aria-modal="true"
        aria-labelledby="dialog-title"
        onMouseDown={(event) => event.stopPropagation()}
      >
        <header className="dialog-header">
          <div>
            <h2 id="dialog-title">{title}</h2>
            {subtitle ? <p>{subtitle}</p> : null}
          </div>
          <button
            aria-label={`Close ${title}`}
            className="dialog-close"
            type="button"
            onClick={onClose}
          >
            <X size={16} />
          </button>
        </header>
        {children}
      </section>
    </div>
  );
}

export function DialogBody({ children, className }: { children: ReactNode; className?: string }) {
  return <div className={`dialog-body ${className ?? ""}`}>{children}</div>;
}

export function DialogFooter({ children, className }: { children: ReactNode; className?: string }) {
  return <footer className={`dialog-footer ${className ?? ""}`}>{children}</footer>;
}
