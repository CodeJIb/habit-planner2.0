import { Check, Circle, Plus, Trash2 } from 'lucide-react'
import { useState } from 'react'
import { Badge } from './ui/badge'
import { Button } from './ui/button'
import { Card } from './ui/card'
import { Input } from './ui/input'
import { Progress } from './ui/progress'
import { ConfirmDialog } from './ConfirmDialog'
import { formatDue, isOverdue } from '../lib/dates'
import type {
  GoalMilestone,
  GoalInput,
  GoalStatus,
  GoalView,
  MilestoneInput,
} from '../types'

const MUTED_SMALL: React.CSSProperties = {
  display: 'block',
  fontSize: '0.75rem',
  color: 'var(--color-muted)',
}

interface Props {
  goals: GoalView[]
  onCreate: (input: GoalInput) => Promise<void>
  onSetStatus: (goal: GoalView, status: GoalStatus) => Promise<void>
  onDelete: (goal: GoalView) => Promise<void>
  onAddMilestone: (goal: GoalView, input: MilestoneInput) => Promise<void>
  onToggleMilestone: (milestone: GoalMilestone) => Promise<void>
  onDeleteMilestone: (milestone: GoalMilestone) => Promise<void>
}

/** Goals with milestones, target dates and completion XP. */
export function GoalsPage({
  goals,
  onCreate,
  onSetStatus,
  onDelete,
  onAddMilestone,
  onToggleMilestone,
  onDeleteMilestone,
}: Props) {
  const [title, setTitle] = useState('')
  const [targetDate, setTargetDate] = useState('')
  const [busy, setBusy] = useState(false)
  const [pendingDelete, setPendingDelete] = useState<GoalView | null>(null)

  async function submit(e: React.FormEvent) {
    e.preventDefault()
    const trimmed = title.trim()
    if (!trimmed || busy) return
    setBusy(true)
    try {
      await onCreate({ title: trimmed, targetDate: targetDate || null })
      setTitle('')
      setTargetDate('')
    } finally {
      setBusy(false)
    }
  }

  return (
    <div className="screen">
      <h1>Goals</h1>
      <p className="sub">Break big goals into milestones. Finish a goal for bonus XP.</p>

      <Card>
        <form onSubmit={submit} className="flex flex-wrap items-center gap-2">
          <Input
            value={title}
            onChange={(e) => setTitle(e.target.value)}
            placeholder="Add a goal…"
            maxLength={200}
            aria-label="Goal title"
            className="min-w-48 flex-1"
          />
          <Input
            type="date"
            value={targetDate}
            onChange={(e) => setTargetDate(e.target.value)}
            aria-label="Target date"
            className="w-44"
          />
          <Button type="submit" disabled={!title.trim() || busy}>
            <Plus className="h-4 w-4" aria-hidden /> Add
          </Button>
        </form>
      </Card>

      <div style={{ display: 'flex', flexDirection: 'column', gap: '0.75rem', marginTop: '1rem' }}>
        {goals.length === 0 ? (
          <p style={{ padding: '2rem 0', textAlign: 'center', color: 'var(--color-muted)' }}>
            No goals yet. Add your first one above.
          </p>
        ) : (
          goals.map((goal) => (
            <GoalCard
              key={goal.id}
              goal={goal}
              onSetStatus={onSetStatus}
              onAddMilestone={onAddMilestone}
              onToggleMilestone={onToggleMilestone}
              onDeleteMilestone={onDeleteMilestone}
              onAskDelete={() => setPendingDelete(goal)}
            />
          ))
        )}
      </div>

      {pendingDelete && (
        <ConfirmDialog
          title={`Delete "${pendingDelete.title}"?`}
          body="This goal and its milestones will be permanently deleted. This cannot be undone."
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

interface CardProps {
  goal: GoalView
  onSetStatus: (goal: GoalView, status: GoalStatus) => Promise<void>
  onAddMilestone: (goal: GoalView, input: MilestoneInput) => Promise<void>
  onToggleMilestone: (milestone: GoalMilestone) => Promise<void>
  onDeleteMilestone: (milestone: GoalMilestone) => Promise<void>
  onAskDelete: () => void
}

function GoalCard({
  goal,
  onSetStatus,
  onAddMilestone,
  onToggleMilestone,
  onDeleteMilestone,
  onAskDelete,
}: CardProps) {
  const [milestoneTitle, setMilestoneTitle] = useState('')
  const [pendingDelete, setPendingDelete] = useState<GoalMilestone | null>(null)

  const completed = goal.status === 'completed'
  const total = goal.milestones.length
  const done = goal.milestones.filter((m) => m.status === 'completed').length
  const pct = total > 0 ? Math.round((done / total) * 100) : 0
  const overdue = !!goal.targetDate && !completed && isOverdue(goal.targetDate)

  async function addMilestone(e: React.FormEvent) {
    e.preventDefault()
    const trimmed = milestoneTitle.trim()
    if (!trimmed) return
    await onAddMilestone(goal, { title: trimmed })
    setMilestoneTitle('')
  }

  return (
    <Card>
      <div style={{ display: 'flex', alignItems: 'center', gap: '0.75rem' }}>
        <button
          type="button"
          onClick={() => onSetStatus(goal, completed ? 'active' : 'completed')}
          aria-pressed={completed}
          aria-label={completed ? `Reopen goal "${goal.title}"` : `Complete goal "${goal.title}"`}
          className="text-muted hover:text-success"
        >
          {completed ? (
            <Check className="h-5 w-5 text-success" />
          ) : (
            <Circle className="h-5 w-5" />
          )}
        </button>

        <div style={{ minWidth: 0, flex: 1 }}>
          <b
            style={{
              textDecoration: completed ? 'line-through' : undefined,
              display: 'block',
            }}
          >
            {goal.title}
          </b>
          <span
            style={{
              ...MUTED_SMALL,
              color: overdue ? 'var(--color-destructive)' : undefined,
            }}
          >
            {goal.targetDate
              ? `${overdue ? 'Overdue · ' : 'Target '}${formatDue(goal.targetDate)}`
              : 'No target date'}
          </span>
        </div>

        {completed && <Badge variant="secondary">Done</Badge>}
        {total > 0 && (
          <span className="xp-pill" aria-label={`${done} of ${total} milestones`}>
            {done}/{total}
          </span>
        )}
        <Button
          variant="outline"
          size="sm"
          onClick={onAskDelete}
          aria-label={`Delete goal "${goal.title}"`}
        >
          <Trash2 aria-hidden />
        </Button>
      </div>

      {total > 0 && (
        <div style={{ marginTop: '0.75rem' }}>
          <Progress value={pct} aria-label={`${goal.title} milestone progress`} />
        </div>
      )}

      {total > 0 && (
        <div style={{ display: 'flex', flexDirection: 'column', gap: '0.25rem' }}>
          {goal.milestones.map((milestone) => {
            const isDone = milestone.status === 'completed'
            return (
              <div
                key={milestone.id}
                style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}
              >
                <button
                  type="button"
                  onClick={() => onToggleMilestone(milestone)}
                  aria-pressed={isDone}
                  aria-label={isDone ? `Reopen milestone "${milestone.title}"` : `Complete milestone "${milestone.title}"`}
                  className="text-muted hover:text-success"
                >
                  {isDone ? (
                    <Check className="h-4 w-4 text-success" />
                  ) : (
                    <Circle className="h-4 w-4" />
                  )}
                </button>
                <span
                  style={{
                    flex: 1,
                    minWidth: 0,
                    textDecoration: isDone ? 'line-through' : undefined,
                    ...MUTED_SMALL,
                    display: 'inline-block',
                    fontSize: '0.875rem',
                  }}
                >
                  {milestone.title}
                </span>
                <button
                  type="button"
                  onClick={() => setPendingDelete(milestone)}
                  aria-label={`Delete milestone "${milestone.title}"`}
                  className="text-muted hover:text-destructive"
                >
                  <Trash2 className="h-4 w-4" aria-hidden />
                </button>
              </div>
            )
          })}
        </div>
      )}

      <form onSubmit={addMilestone} className="flex gap-2" style={{ marginTop: '0.75rem' }}>
        <Input
          value={milestoneTitle}
          onChange={(e) => setMilestoneTitle(e.target.value)}
          placeholder="Add a milestone…"
          maxLength={200}
          aria-label="Milestone title"
          className="min-w-0 flex-1"
        />
        <Button type="submit" variant="outline" disabled={!milestoneTitle.trim()}>
          <Plus className="h-4 w-4" aria-hidden /> Add
        </Button>
      </form>

      {pendingDelete && (
        <ConfirmDialog
          title={`Delete "${pendingDelete.title}"?`}
          body="This milestone will be permanently deleted. This cannot be undone."
          onCancel={() => setPendingDelete(null)}
          onConfirm={() => {
            onDeleteMilestone(pendingDelete)
            setPendingDelete(null)
          }}
        />
      )}
    </Card>
  )
}
