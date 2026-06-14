/**
 * Global keyboard shortcut hook.
 *
 * Registers key bindings on the window and cleans up on unmount.
 * Binding keys follow the format: "ctrl+k", "meta+k", "escape", etc.
 */

import { useEffect } from "react";

type HotkeyMap = Record<string, () => void>;

function normalizeKey(event: KeyboardEvent): string {
  const parts: string[] = [];
  if (event.ctrlKey) parts.push("ctrl");
  if (event.metaKey) parts.push("meta");
  if (event.altKey) parts.push("alt");
  if (event.shiftKey) parts.push("shift");
  parts.push(event.key.toLowerCase());
  return parts.join("+");
}

/**
 * Register global keyboard shortcuts.
 *
 * @example
 * useHotkeys({
 *   "ctrl+k": () => openSearch(),
 *   "meta+k": () => openSearch(),
 *   "escape": () => closeModal(),
 * });
 */
export function useHotkeys(bindings: HotkeyMap): void {
  useEffect(() => {
    const keys = Object.keys(bindings);
    if (keys.length === 0) return;

    function handler(event: KeyboardEvent) {
      // Don't intercept shortcuts when typing in inputs/textareas
      const tag = (event.target as HTMLElement)?.tagName;
      const isInput = tag === "INPUT" || tag === "TEXTAREA" || tag === "SELECT";

      const normalized = normalizeKey(event);
      const action = bindings[normalized];

      if (action) {
        // Allow Escape even in inputs
        if (isInput && event.key.toLowerCase() !== "escape") return;
        event.preventDefault();
        action();
      }
    }

    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [bindings]);
}
