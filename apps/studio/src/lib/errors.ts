/**
 * Error normalization for user-facing display.
 *
 * Strips sensitive details like full filesystem paths and stack traces
 * while preserving the meaningful error message for the user.
 */

/** Patterns that indicate sensitive path information. */
const PATH_PATTERNS = [
  /[A-Z]:\\[^ ]+/gi,           // Windows absolute paths
  /\/(?:Users|home|root)\/[^ ]+/gi, // Unix home paths
  /at\s+\S+\s+\(\S+:\d+:\d+\)/g,   // Stack trace frames
];

/**
 * Convert any thrown value into a safe, concise user-facing string.
 */
export function safeErrorMessage(error: unknown): string {
  if (error === null || error === undefined) {
    return "An unknown error occurred";
  }

  let message: string;

  if (typeof error === "string") {
    message = error;
  } else if (error instanceof Error) {
    message = error.message;
  } else if (typeof error === "object" && "message" in error && typeof (error as { message: unknown }).message === "string") {
    message = (error as { message: string }).message;
  } else {
    message = String(error);
  }

  // Strip sensitive paths but keep the rest of the message intact
  for (const pattern of PATH_PATTERNS) {
    message = message.replace(pattern, "[path]");
  }

  // Trim and limit length for display
  message = message.trim();
  if (message.length > 300) {
    message = `${message.slice(0, 297)}...`;
  }

  return message || "An unknown error occurred";
}

/**
 * Check whether an error value contains a meaningful user-facing message.
 */
export function isUserFacingError(error: unknown): boolean {
  if (typeof error === "string" && error.trim().length > 0) return true;
  if (error instanceof Error && error.message.trim().length > 0) return true;
  return false;
}
