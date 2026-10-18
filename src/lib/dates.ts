const pad = (n: number) => String(n).padStart(2, '0')

/** Format a Date as YYYY-MM-DD in the user's LOCAL timezone (toISOString would give UTC). */
export function formatLocalDate(d: Date): string {
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`
}

/** Today's date in the user's LOCAL timezone as YYYY-MM-DD. */
export function localToday(): string {
  return formatLocalDate(new Date())
}

/**
 * True when a due moment is in the past. Accepts "YYYY-MM-DD" (overdue once the
 * day has passed) or "YYYY-MM-DDTHH:MM" (overdue once the clock passes that minute).
 */
export function isOverdue(dueAt: string | null): boolean {
  if (!dueAt) return false
  const date = dueAt.slice(0, 10)
  const today = localToday()
  if (date !== today) return date < today
  if (dueAt.length > 10) {
    const now = new Date()
    return dueAt.slice(11, 16) < `${pad(now.getHours())}:${pad(now.getMinutes())}`
  }
  return false
}

export function formatDue(dueAt: string): string {
  const [y, m, d] = dueAt.slice(0, 10).split('-').map(Number)
  return new Date(y, m - 1, d).toLocaleDateString(undefined, { month: 'short', day: 'numeric' })
}

/** Add `days` to a "YYYY-MM-DD" date, returning "YYYY-MM-DD" in local time. */
export function addDays(dateStr: string, days: number): string {
  const [y, m, d] = dateStr.split('-').map(Number)
  const dt = new Date(y, m - 1, d)
  dt.setDate(dt.getDate() + days)
  return formatLocalDate(dt)
}

/** The Monday of the week containing `dateStr` (both "YYYY-MM-DD"). */
export function mondayOf(dateStr: string): string {
  const [y, m, d] = dateStr.split('-').map(Number)
  const dt = new Date(y, m - 1, d)
  const offset = (dt.getDay() + 6) % 7
  dt.setDate(dt.getDate() - offset)
  return formatLocalDate(dt)
}
