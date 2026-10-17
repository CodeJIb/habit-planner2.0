import { ChevronLeft, ChevronRight, Plus, Trash2 } from 'lucide-react'
import { useState } from 'react'
import { Badge } from './ui/badge'
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
import { ConfirmDialog } from './ConfirmDialog'
import { addDays, localToday, mondayOf } from '../lib/dates'
import type {
  BlockStatus,
  BlockType,
  ScheduledBlock,
  ScheduledBlockInput,
  Task,
} from '../types'

const BLOCK_LABEL: Record<BlockType, string> = {
  task: 'Task',
  break: 'Break',
  other: 'Other',
}

const BLOCK_VARIANT: Record<BlockType, 'default' | 'secondary' | 'outline'> = {
  task: 'default',
  break: 'secondary',
  other: 'outline',
}

interface Props {
  blocks: ScheduledBlock[]
  tasks: Task[]
  onCreate: (input: ScheduledBlockInput) => Promise<void>
  onSetStatus: (block: ScheduledBlock, status: BlockStatus) => Promise<void>
  onDelete: (block: ScheduledBlock) => Promise<void>
}

/** "2026-10-16" -> "Fri, Oct 16" for the day header. */
function dayLabel(date: string): string {
  const [y, m, d] = date.split('-').map(Number)
  return new Date(y, m - 1, d).toLocaleDateString(undefined, {
    weekday: 'short',
    month: 'short',
    day: 'numeric',
  })
}

/** Blocks that overlap the given day (a block spanning midnight counts too). */
function blocksForDay(blocks: ScheduledBlock[], day: string): ScheduledBlock[] {
  return blocks
    .filter((b) => b.startAt.slice(0, 10) <= day && b.endAt.slice(0, 10) >= day)
    .sort((a, b) => a.startAt.localeCompare(b.startAt))
}

/** "2026-10-16T09:00" -> "09:00". */
function timeOf(iso: string): string {
  return iso.slice(11, 16)
}

/** The Calendar screen: pick a day, see its blocks, schedule new ones. */
export function CalendarPage({ blocks, tasks, onCreate, onSetStatus, onDelete }: Props) {
  const today = localToday()
  const [weekStart, setWeekStart] = useState(() => mondayOf(today))
  const [selectedDay, setSelectedDay] = useState(today)

  const [title, setTitle] = useState('')
  const [startTime, setStartTime] = useState('09:00')
  const [endTime, setEndTime] = useState('10:00')
  const [blockType, setBlockType] = useState<BlockType>('task')
  const [taskId, setTaskId] = useState<number | null>(null)
  const [busy, setBusy] = useState(false)
  const [pendingDelete, setPendingDelete] = useState<ScheduledBlock | null>(null)

  const week = Array.from({ length: 7 }, (_, i) => addDays(weekStart, i))
  const dayBlocks = blocksForDay(blocks, selectedDay)

  function shiftWeek(days: number) {
    setWeekStart((prev) => addDays(prev, days))
    setSelectedDay((prev) => addDays(prev, days))
  }

  function goToday() {
    setWeekStart(mondayOf(today))
    setSelectedDay(today)
  }

  async function submit(e: React.FormEvent) {
    e.preventDefault()
    const trimmed = title.trim()
    if (!trimmed || busy) return
    setBusy(true)
    try {
      await onCreate({
        title: trimmed,
        startAt: `${selectedDay}T${startTime}`,
        endAt: `${selectedDay}T${endTime}`,
        taskId,
        blockType,
      })
      setTitle('')
    } finally {
      setBusy(false)
    }
  }

  return (
    <div className="screen">
      <h1>Calendar</h1>
      <p className="sub">Plan your day in time blocks. A block can be tied to a task.</p>

      <Card>
        <form onSubmit={submit} className="flex flex-wrap items-end gap-2">
          <label className="flex flex-col gap-1">
            <span className="small">Title</span>
            <Input
              value={title}
              onChange={(e) => setTitle(e.target.value)}
              placeholder="Block title…"
              maxLength={200}
              aria-label="Block title"
              className="min-w-40 flex-1"
            />
          </label>
          <label className="flex flex-col gap-1">
            <span className="small">Start</span>
            <Input
              type="time"
              value={startTime}
              onChange={(e) => setStartTime(e.target.value)}
              aria-label="Start time"
              className="w-32"
            />
          </label>
          <label className="flex flex-col gap-1">
            <span className="small">End</span>
            <Input
              type="time"
              value={endTime}
              onChange={(e) => setEndTime(e.target.value)}
              aria-label="End time"
              className="w-32"
            />
          </label>
          <label className="flex flex-col gap-1">
            <span className="small">Type</span>
            <Select value={blockType} onValueChange={(v) => setBlockType(v as BlockType)}>
              <SelectTrigger aria-label="Block type" className="w-32">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="task">Task</SelectItem>
                <SelectItem value="break">Break</SelectItem>
                <SelectItem value="other">Other</SelectItem>
              </SelectContent>
            </Select>
          </label>
          <label className="flex flex-col gap-1">
            <span className="small">Task</span>
            <Select
              value={taskId === null ? 'none' : String(taskId)}
              onValueChange={(v) => {
                if (v === 'none') {
                  setTaskId(null)
                } else {
                  const id = Number(v)
                  setTaskId(id)
                  const task = tasks.find((t) => t.id === id)
                  if (task) setTitle(task.title)
                }
              }}
            >
              <SelectTrigger aria-label="Link a task" className="w-40">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="none">None</SelectItem>
                {tasks.map((task) => (
                  <SelectItem key={task.id} value={String(task.id)}>
                    {task.title}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </label>
          <Button type="submit" disabled={!title.trim() || busy}>
            <Plus className="h-4 w-4" aria-hidden /> Add
          </Button>
        </form>
      </Card>

      <div
        style={{
          display: 'flex',
          alignItems: 'center',
          gap: '0.5rem',
          marginTop: '1.25rem',
          marginBottom: '0.75rem',
        }}
      >
        <Button
          variant="outline"
          size="sm"
          onClick={() => shiftWeek(-1)}
          aria-label="Previous week"
        >
          <ChevronLeft aria-hidden />
        </Button>
        <b style={{ minWidth: '9rem', textAlign: 'center' }}>{dayLabel(weekStart)}</b>
        <Button variant="outline" size="sm" onClick={() => shiftWeek(1)} aria-label="Next week">
          <ChevronRight aria-hidden />
        </Button>
        <div className="grow" />
        <Button variant="ghost" size="sm" onClick={goToday}>
          Today
        </Button>
      </div>

      <div className="week" aria-label="Days of the week">
        {week.map((date) => {
          const isToday = date === today
          const [y, m, d] = date.split('-').map(Number)
          return (
            <button
              key={date}
              type="button"
              aria-pressed={date === selectedDay}
              onClick={() => setSelectedDay(date)}
              aria-label={`Show ${dayLabel(date)}`}
              title={dayLabel(date)}
              style={{ width: '2.5rem', height: '2.5rem', fontSize: '0.75rem' }}
            >
              {new Date(y, m - 1, d).getDate()}
              {isToday ? '•' : ''}
            </button>
          )
        })}
      </div>

      <h2 style={{ marginTop: '1.25rem' }}>{dayLabel(selectedDay)}</h2>

      {dayBlocks.length === 0 ? (
        <p style={{ padding: '2rem 0', textAlign: 'center', color: 'var(--color-muted)' }}>
          No blocks scheduled for this day.
        </p>
      ) : (
        <div style={{ display: 'flex', flexDirection: 'column', gap: '0.25rem' }}>
          {dayBlocks.map((block) => (
            <Card key={block.id} className="row">
              <span className="xp-pill" style={{ minWidth: '4.5rem' }}>
                {timeOf(block.startAt)}–{timeOf(block.endAt)}
              </span>
              <div className="grow" style={{ minWidth: 0 }}>
                <b>{block.title}</b>
                {block.taskId !== null && <span className="small">Linked to a task</span>}
              </div>
              <Badge variant={BLOCK_VARIANT[block.blockType]}>
                {BLOCK_LABEL[block.blockType]}
              </Badge>
              <Select
                value={block.status}
                onValueChange={(v) => void onSetStatus(block, v as BlockStatus)}
              >
                <SelectTrigger aria-label={`Status for ${block.title}`} className="w-32" size="sm">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="scheduled">Scheduled</SelectItem>
                  <SelectItem value="running">Running</SelectItem>
                  <SelectItem value="done">Done</SelectItem>
                  <SelectItem value="cancelled">Cancelled</SelectItem>
                </SelectContent>
              </Select>
              <Button
                variant="outline"
                size="sm"
                onClick={() => setPendingDelete(block)}
                aria-label={`Delete block "${block.title}"`}
              >
                <Trash2 aria-hidden />
              </Button>
            </Card>
          ))}
        </div>
      )}

      {pendingDelete && (
        <ConfirmDialog
          title={`Delete "${pendingDelete.title}"?`}
          body="This block will be permanently deleted. This cannot be undone."
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
