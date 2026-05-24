export function formatError(err: unknown, fallback: string): string {
  if (typeof err === 'string') return err
  if (err instanceof Error && err.message) return err.message
  return fallback
}
