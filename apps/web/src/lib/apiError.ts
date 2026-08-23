/** Read a server `ErrorBody.message` from an openapi-ts error payload. */
export function readApiErrorMessage(error: unknown, fallback: string): string {
  if (!error || typeof error !== 'object') {
    return fallback;
  }
  const body = error as {
    message?: string;
    details?: { fields?: Record<string, string[]> };
  };
  const message = body.message ? String(body.message) : fallback;
  const fields = body.details?.fields;
  if (!fields) {
    return message;
  }
  const fieldNotes = Object.entries(fields)
    .map(([name, issues]) => `${name}: ${issues.join(', ')}`)
    .join('; ');
  return fieldNotes ? `${message} (${fieldNotes})` : message;
}
