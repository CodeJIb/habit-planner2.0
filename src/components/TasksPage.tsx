import { Check, Circle, Plus, Trash2 } from 'lucide-react'
import { useState } from 'react'
import { Button } from './ui/button'
import { Card } from './ui/card'
import { Input } from './ui/input'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from './ui/select'
import { Badge } from './ui/badge'
import { ConfirmDialog } from './ConfirmDialog'
import { isOverdue } from '../lib/dates'
import type { Priority, Task, TaskInput } from '../types'

interface Props {
  tasks: Task[]
  onCreate: (input: TaskInput) => Promise<void>
  onToggle: (task: Task) => Promise<void>
  onDelete: (task: Task) => Promise<void>
}

const PRIORITY_VARIANT: Record<Priority, 'default' | 'secondary' | 'outline' | 'link'> = {
  low: 'secondary',
  medium: 'default',
  high: 'link',
  urgent: 'link',
}

export function TasksPage({ tasks, onCreate, onToggle, onDelete }: Props) {
  const [title, setTitle] = useState('')
  const [priority, setPriority] = useState<Priority>('medium')
  const [dueAt, setDueAt] = useState('')
  const [busy, setBusy] = useState(false)
  const [pendingDelete, setPendingDelete] = useState<Task | null>(null)

  const sorted = [...tasks].sort((a, b) => {
    const da = a.dueAt ? a.dueAt.slice(0, 10) : '9999-12-31'
    const db = b.dueAt ? b.dueAt.slice(0, 10) : '9999-12-31'
    return da.localeCompare(db)
  })

  async function submit(e: React.FormEvent) {
    e.preventDefault()
    const trimmed = title.trim()
    if (!trimmed || busy) return
    setBusy(true)
    try {
      await onCreate({ title: trimmed, priority, dueAt: dueAt || null })
      setTitle('')
      setDueAt('')
    } finally {
      setBusy(false)
    }
  }

  return (
    <div>
      <h1>Tasks</h1>
      <p className="sub">Complete a task to earn XP. The highest priority first.</p>

      <Card>
        <form onSubmit={submit} className="flex flex-wrap items-center gap-2">
          <Input
            value={title}
            onChange={(e) => setTitle(e.target.value)}
            placeholder="Add a task…"
            maxLength={200}
            aria-label="Task title"
            className="min-w-48 flex-1"
          />
          <Select value={priority} onValueChange={(v) => setPriority(v as Priority)}>
            <SelectTrigger aria-label="Priority" className="w-36">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="low">Low</SelectItem>
              <SelectItem value="medium">Medium</SelectItem>
              <SelectItem value="high">High</SelectItem>
              <SelectItem value="urgent">Urgent</SelectItem>
            </SelectContent>
          </Select>
          <Input
            type="date"
            value={dueAt}
            onChange={(e) => setDueAt(e.target.value)}
            aria-label="Due date"
            className="w-44"
          />
          <Button type="submit" disabled={!title.trim() || busy}>
            <Plus className="h-4 w-4" aria-hidden /> Add
          </Button>
        </form>
      </Card>

      <div style={{ display: 'flex', flexDirection: 'column', gap: '0.25rem', marginTop: '1rem' }}>
        {sorted.length === 0 ? (
          <p style={{ padding: '2rem 0', textAlign: 'center', color: 'var(--color-muted)' }}>
            No tasks yet. Add your first one above.
          </p>
        ) : (
          sorted.map((task) => (
            <Card key={task.id} className="row" >
              <button
                type="button"
                onClick={() => onToggle(task)}
                aria-pressed={task.status === 'done'}
                aria-label={task.status === 'done' ? `Reopen "${task.title}"` : `Complete "${task.title}"`}
                className="text-muted hover:text-success"
              >
                {task.status === 'done' ? (
                  <Check className="h-5 w-5 text-success" />
                ) : (
                  <Circle className="h-5 w-5" />
                )}
              </button>
              <div className="grow" style={{ minWidth: 0 }}>
                <b style={{ textDecoration: task.status === 'done' ? 'line-through' : undefined }}>
                  {task.title}
                </b>
                {task.dueAt !== null && (
                  <span
                    className="small"
                    style={
                      isOverdue(task.dueAt) ? { color: 'var(--color-destructive)' } : undefined
                    }
                  >
                    {isOverdue(task.dueAt) ? 'Overdue · ' : 'Due '}
                    {task.dueAt.slice(0, 10)}
                  </span>
                )}
              </div>
              <Badge variant={PRIORITY_VARIANT[task.priority]}>{task.priority}</Badge>
              <Button
                variant="outline"
                size="sm"
                onClick={() => setPendingDelete(task)}
                aria-label={`Delete "${task.title}"`}
              >
                <Trash2 aria-hidden />
              </Button>
            </Card>
          ))
        )}
      </div>

      {pendingDelete && (
        <ConfirmDialog
          title={`Delete "${pendingDelete.title}"?`}
          body="This task will be permanently deleted. This cannot be undone."
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
