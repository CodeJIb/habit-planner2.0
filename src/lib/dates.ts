const pad = (n: number) => String(n).padStart(2, '0')

/** Today's date in the user's LOCAL timezone as YYYY-MM-DD (toISOString would give UTC). */
export function localToday(): string {
  const d = new Date()
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`
}

/** True when a due date (YYYY-MM-DD or ISO string) is before today. */
export function isOverdue(dueAt: string | null, today = localToday()): boolean {
  if (!dueAt) return false
  return dueAt.slice(0, 10) < today
}

export function formatDue(dueAt: string): string {
  const [y, m, d] = dueAt.slice(0, 10).split('-').map(Number)
  return new Date(y, m - 1, d).toLocaleDateString(undefined, { month: 'short', day: 'numeric' })
}
