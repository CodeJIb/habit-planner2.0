import { Pause, Play, Square, Trash2 } from 'lucide-react'
import { useEffect, useState } from 'react'
import { Badge } from './ui/badge'
import { Button } from './ui/button'
import { Card } from './ui/card'
import { Progress } from './ui/progress'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from './ui/select'
import { ConfirmDialog } from './ConfirmDialog'
import { FocusMusic } from './FocusMusic'
import type { FocusSession, FocusSessionInput, FocusStats, Task } from '../types'

const DURATIONS = [15, 25, 45, 60]

interface Props {
  sessions: FocusSession[]
  stats: FocusStats | null
  tasks: Task[]
  onStart: (input: FocusSessionInput) => Promise<FocusSession | null>
  onComplete: (id: number, actualDurationSeconds: number) => Promise<void>
  onCancel: (id: number) => Promise<void>
  onDelete: (session: FocusSession) => Promise<void>
}

function formatClock(seconds: number): string {
  const mm = String(Math.floor(seconds / 60)).padStart(2, '0')
  const ss = String(seconds % 60).padStart(2, '0')
  return `${mm}:${ss}`
}

/** Seconds a completed session focused, for the history line. */
function focusedMinutes(session: FocusSession): number {
  return Math.round((session.actualDurationSeconds ?? 0) / 60)
}

/** A Pomodoro timer with a running session log and daily/weekly totals. */
export function FocusPage({ sessions, stats, tasks, onStart, onComplete, onCancel, onDelete }: Props) {
  const [duration, setDuration] = useState(25)
  const [taskId, setTaskId] = useState<number | null>(null)
  const [session, setSession] = useState<FocusSession | null>(null)
  const [secondsLeft, setSecondsLeft] = useState(duration * 60)
  const [running, setRunning] = useState(false)
  const [busy, setBusy] = useState(false)
  const [pendingDelete, setPendingDelete] = useState<FocusSession | null>(null)

  // Tick the countdown while a session is running.
  useEffect(() => {
    if (!running || !session) return
    const id = window.setInterval(() => setSecondsLeft((s) => Math.max(0, s - 1)), 1000)
    return () => window.clearInterval(id)
  }, [running, session])

  // Auto-finish the moment the countdown hits zero.
  useEffect(() => {
    if (!running || secondsLeft > 0 || !session) return
    // oxlint-disable-next-line react/set-state-in-effect
    setRunning(false)
    const focused = Math.max(0, session.plannedDurationMinutes * 60 - secondsLeft)
    void onComplete(session.id, focused).finally(() => {
      setSession(null)
      setSecondsLeft(duration * 60)
    })
  }, [running, secondsLeft, session, duration, onComplete])

  async function start() {
    if (busy || session) return
    setBusy(true)
    try {
      const created = await onStart({ taskId, plannedDurationMinutes: duration })
      if (created) {
        setSession(created)
        setSecondsLeft(created.plannedDurationMinutes * 60)
        setRunning(true)
      }
    } finally {
      setBusy(false)
    }
  }

  async function stop() {
    if (busy || !session) return
    setBusy(true)
    try {
      await onCancel(session.id)
    } finally {
      setRunning(false)
      setSession(null)
      setSecondsLeft(duration * 60)
      setBusy(false)
    }
  }

  /** One play/pause control for the timer and its music; starts a session if idle. */
  function togglePlay() {
    if (busy) return
    if (!session) void start()
    else setRunning((r) => !r)
  }

  function taskTitle(id: number | null): string {
    if (id === null) return 'No task'
    return tasks.find((t) => t.id === id)?.title ?? 'Deleted task'
  }

  const plannedSeconds = session ? session.plannedDurationMinutes * 60 : duration * 60
  const pct = plannedSeconds > 0 ? Math.round(((plannedSeconds - secondsLeft) / plannedSeconds) * 100) : 0

  return (
    <div className="screen">
      <h1>Focus</h1>
      <p className="sub">Run a Pomodoro, stay on task, and log every focused minute.</p>

      <div style={{ display: 'flex', flexWrap: 'wrap', gap: '1.5rem', marginBottom: '1rem' }}>
        <span className="xp-pill" aria-label="Focus time today">
          Today · {stats?.todayMinutes ?? 0} min ({stats?.todaySessions ?? 0})
        </span>
        <span className="xp-pill" aria-label="Focus time this week">
          This week · {stats?.weekMinutes ?? 0} min ({stats?.weekSessions ?? 0})
        </span>
      </div>

      <Card>
        <div
          style={{
            textAlign: 'center',
            fontFamily: 'var(--font-display)',
            fontSize: '3.5rem',
            fontWeight: 700,
            fontVariantNumeric: 'tabular-nums',
            lineHeight: 1.1,
          }}
          aria-live="polite"
        >
          {formatClock(secondsLeft)}
        </div>

        <div style={{ margin: '0.75rem 0 1.25rem' }}>
          <Progress value={pct} aria-label="Session progress" />
        </div>

        <div className="flex flex-wrap items-center justify-center gap-2">
          <label className="flex items-center gap-2">
            <span className="small">Length</span>
            <Select
              value={String(duration)}
              onValueChange={(v) => {
                const minutes = Number(v)
                setDuration(minutes)
                if (!session) setSecondsLeft(minutes * 60)
              }}
              disabled={!!session}
            >
              <SelectTrigger aria-label="Session length" className="w-28" size="sm">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {DURATIONS.map((minutes) => (
                  <SelectItem key={minutes} value={String(minutes)}>
                    {minutes} min
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </label>

          <label className="flex items-center gap-2">
            <span className="small">Task</span>
            <Select
              value={taskId === null ? 'none' : String(taskId)}
              onValueChange={(v) => setTaskId(v === 'none' ? null : Number(v))}
              disabled={!!session}
            >
              <SelectTrigger aria-label="Link a task" className="w-44" size="sm">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="none">No task</SelectItem>
                {tasks.map((task) => (
                  <SelectItem key={task.id} value={String(task.id)}>
                    {task.title}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </label>
        </div>

        <div className="flex items-center justify-center gap-2" style={{ marginTop: '1.25rem' }}>
          {!session ? (
            <Button onClick={start} disabled={busy}>
              <Play className="h-4 w-4" aria-hidden /> Start
            </Button>
          ) : (
            <>
              <Button onClick={() => setRunning((r) => !r)} disabled={busy}>
                {running ? (
                  <>
                    <Pause className="h-4 w-4" aria-hidden /> Pause
                  </>
                ) : (
                  <>
                    <Play className="h-4 w-4" aria-hidden /> Resume
                  </>
                )}
              </Button>
              <Button variant="outline" onClick={stop} disabled={busy}>
                <Square className="h-4 w-4" aria-hidden /> Give up
              </Button>
            </>
          )}
        </div>
      </Card>

      <div style={{ marginTop: '1rem' }}>
        <FocusMusic playing={running && !!session} onTogglePlay={togglePlay} busy={busy} />
      </div>

      <h2 style={{ marginTop: '1.5rem' }}>History</h2>

      {sessions.length === 0 ? (
        <p style={{ padding: '2rem 0', textAlign: 'center', color: 'var(--color-muted)' }}>
          No focus sessions yet. Start the timer above.
        </p>
      ) : (
        <div style={{ display: 'flex', flexDirection: 'column', gap: '0.25rem' }}>
          {sessions.map((item) => {
            const completed = item.status === 'completed'
            return (
              <Card key={item.id} className="row">
                <span className="xp-pill" style={{ minWidth: '3.5rem' }}>
                  {item.startedAt.slice(11, 16)}
                </span>
                <div className="grow" style={{ minWidth: 0 }}>
                  <b>{taskTitle(item.taskId)}</b>
                  <span className="small">
                    {completed
                      ? `Focused ${focusedMinutes(item)} of ${item.plannedDurationMinutes} min`
                      : `Planned ${item.plannedDurationMinutes} min`}
                  </span>
                </div>
                <Badge variant={completed ? 'secondary' : 'outline'}>
                  {completed ? 'Completed' : 'Cancelled'}
                </Badge>
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() => setPendingDelete(item)}
                  aria-label={`Delete session at ${item.startedAt.slice(11, 16)}`}
                >
                  <Trash2 aria-hidden />
                </Button>
              </Card>
            )
          })}
        </div>
      )}

      {pendingDelete && (
        <ConfirmDialog
          title="Delete this session?"
          body="This focus session will be permanently deleted and any XP it earned is removed. This cannot be undone."
          onCancel={() => setPendingDelete(null)}
          onConfirm={() => {
            onDelete(pendingDelete)
            setPendingDelete(null)
          }}
        />
      )}
    </div>
  )
}
