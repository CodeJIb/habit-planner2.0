import { Check, Circle, Flame, Plus, Trash2 } from 'lucide-react'
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
import { ConfirmDialog } from './ConfirmDialog'
import { formatLocalDate, localToday } from '../lib/dates'
import type { Frequency, HabitInput, HabitView } from '../types'

interface Props {
  habits: HabitView[]
  onCreate: (input: HabitInput) => Promise<void>
  onToggle: (habit: HabitView) => Promise<void>
  onDelete: (habit: HabitView) => Promise<void>
}

/** The seven day buttons of the current week, in calendar order. */
function weekDays(): { label: string; date: string }[] {
  const today = localToday()
  const start = new Date(`${today}T00:00:00`)
  // Monday-first week containing today.
  const offset = (start.getDay() + 6) % 7
  start.setDate(start.getDate() - offset)
  const labels = ['M', 'T', 'W', 'T', 'F', 'S', 'S']
  return Array.from({ length: 7 }, (_, i) => {
    const day = new Date(start)
    day.setDate(start.getDate() + i)
    return { label: labels[i], date: formatLocalDate(day) }
  })
}

export function HabitsPage({ habits, onCreate, onToggle, onDelete }: Props) {
  const [title, setTitle] = useState('')
  const [frequency, setFrequency] = useState<Frequency>('daily')
  const [reminder, setReminder] = useState('')
  const [busy, setBusy] = useState(false)
  const [pendingDelete, setPendingDelete] = useState<HabitView | null>(null)

  async function submit(e: React.FormEvent) {
    e.preventDefault()
    const trimmed = title.trim()
    if (!trimmed || busy) return
    setBusy(true)
    try {
      await onCreate({
        title: trimmed,
        frequencyType: frequency,
        reminderTime: reminder || null,
      })
      setTitle('')
      setReminder('')
    } finally {
      setBusy(false)
    }
  }

  const week = weekDays()
  const todayIso = localToday()

  return (
    <div>
      <h1>Habits</h1>
      <p className="sub">Check in once a day to keep your streak. The whole week is shown.</p>

      <Card>
        <form onSubmit={submit} className="flex flex-wrap items-center gap-2">
          <Input
            value={title}
            onChange={(e) => setTitle(e.target.value)}
            placeholder="New habit…"
            maxLength={200}
            aria-label="Habit title"
            className="min-w-48 flex-1"
          />
          <Select value={frequency} onValueChange={(v) => setFrequency(v as Frequency)}>
            <SelectTrigger aria-label="Frequency" className="w-40">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="daily">Daily</SelectItem>
              <SelectItem value="weekly">Weekly</SelectItem>
              <SelectItem value="custom">Custom</SelectItem>
            </SelectContent>
          </Select>
          <Input
            type="time"
            value={reminder}
            onChange={(e) => setReminder(e.target.value)}
            aria-label="Reminder time"
            title="Optional: send a phone reminder at this time"
            className="w-32"
          />
          <Button type="submit" disabled={!title.trim() || busy}>
            <Plus className="h-4 w-4" aria-hidden /> Add
          </Button>
        </form>
      </Card>

      <div style={{ display: 'flex', flexDirection: 'column', gap: '0.25rem', marginTop: '1rem' }}>
        {habits.length === 0 ? (
          <p style={{ padding: '2rem 0', textAlign: 'center', color: 'var(--color-muted)' }}>
            No habits yet. Add your first one above.
          </p>
        ) : (
          habits.map((habit) => (
            <Card key={habit.id} className="row" >
              <button
                type="button"
                onClick={() => onToggle(habit)}
                aria-pressed={habit.completedToday}
                aria-label={habit.completedToday ? `Undo check-in: ${habit.title}` : `Check in today: ${habit.title}`}
                className="text-muted hover:text-success"
              >
                {habit.completedToday ? (
                  <Check className="h-5 w-5 text-success" />
                ) : (
                  <Circle className="h-5 w-5" />
                )}
              </button>
              <div className="grow" style={{ minWidth: 0 }}>
                <b>{habit.title}</b>
                <span className="small">
                  Current streak: {habit.currentStreak} day{habit.currentStreak === 1 ? '' : 's'}
                  {habit.reminderTime ? ` · Reminder at ${habit.reminderTime}` : ''}
                </span>
                <div className="week" aria-label={`This week for ${habit.title}`}>
                  {week.map((day) => {
                    const done = day.date === todayIso && habit.completedToday
                    const isToday = day.date === todayIso
                    return (
                      <button
                        key={day.date}
                        type="button"
                        disabled={!isToday}
                        aria-pressed={done}
                        aria-label={
                          isToday
                            ? done
                              ? `Checked in today: ${habit.title}`
                              : `Check in ${habit.title} today`
                            : undefined
                        }
                        title={isToday ? (done ? 'Checked in today' : 'Check in today') : 'Only today can be checked'}
                      >
                        {day.label}
                      </button>
                    )
                  })}
                </div>
              </div>
              <span className="xp-pill">
                <Flame aria-hidden className="mr-1 inline h-3 w-3" />
                {habit.currentStreak}
              </span>
              <Button
                variant="outline"
                size="sm"
                onClick={() => setPendingDelete(habit)}
                aria-label={`Delete "${habit.title}"`}
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
          body="This habit will be permanently deleted. This cannot be undone."
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
