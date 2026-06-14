/**
 * Typed wrapper around Tauri's invoke API.
 *
 * All frontend → backend calls go through this module so that:
 *  1. Error messages are normalized to safe user-facing strings.
 *  2. Sensitive details (full paths, stack traces) are stripped.
 *  3. Every call is typed at the call-site.
 */

import { invoke } from "@tauri-apps/api/core";
import { safeErrorMessage } from "./errors";

/**
 * Invoke a Tauri command with typed return value and safe error handling.
 */
export async function tauriInvoke<T>(
  command: string,
  args?: Record<string, unknown>,
): Promise<T> {
  try {
    return await invoke<T>(command, args);
  } catch (error) {
    throw new Error(safeErrorMessage(error));
  }
}
