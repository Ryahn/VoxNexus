/** Read a server `ErrorBody.message` from an openapi-ts error payload. */
export function readApiErrorMessage(error: unknown, fallback: string): string {
  if (error && typeof error === 'object' && 'message' in error) {
    return String((error as { message: string }).message);
  }
  return fallback;
}
