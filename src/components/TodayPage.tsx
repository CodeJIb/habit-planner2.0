import { Sparkles } from 'lucide-react'
import { useState } from 'react'
import { Progress } from './ui/progress'
import type { HabitView, PlayerProfile, Task } from '../types'
import { localToday } from '../lib/dates'

interface Props {
  tasks: Task[]
  habits: HabitView[]
  profile: PlayerProfile
}

function greetingFor(hour: number): string {
  if (hour < 12) return 'Good morning'
  if (hour < 18) return 'Good afternoon'
  return 'Good evening'
}

/** Open tasks: everything not done yet, with overdue tasks first. */
function todaysOpen(tasks: Task[]): Task[] {
  const open = tasks.filter((t) => t.status !== 'done' && t.status !== 'archived')
  return open.sort((a, b) => {
    const da = a.dueAt ? a.dueAt.slice(0, 10) : '9999'
    const db = b.dueAt ? b.dueAt.slice(0, 10) : '9999'
    return da.localeCompare(db)
  })
}

export function TodayPage({ tasks, habits, profile }: Props) {
  // Greeting and date are captured once so they stay stable across re-renders.
  const [header] = useState(() => {
    const now = new Date()
    return {
      greeting: greetingFor(now.getHours()),
      date: now.toLocaleDateString(undefined, { weekday: 'long', day: 'numeric', month: 'long' }),
    }
  })
  const openTasks = todaysOpen(tasks)
  const openHabits = habits.filter((h) => !h.completedToday)
  const best = bestStreak(habits)
  const pct = Math.min(100, Math.round((profile.xpIntoLevel / profile.xpForNext) * 100))
  const circumference = 2 * Math.PI * 40

  return (
    <div className="screen">
      <h1>{header.greeting}</h1>
      <p className="sub">
        {header.date} ·{' '}
        {openTasks.length} tasks and {openHabits.length} habits left to check in.
      </p>

      <div className="card" aria-label="Level progress">
        <div style={{ display: 'flex', alignItems: 'center', gap: '1.25rem', flexWrap: 'wrap' }}>
          <div
            style={{
              position: 'relative',
              width: '7rem',
              height: '7rem',
              flex: 'none',
              borderRadius: '9999px',
            }}
          >
            <svg viewBox="0 0 100 100" width="100" height="100" aria-hidden>
              <circle
                cx="50"
                cy="50"
                r="40"
                fill="none"
                stroke="var(--color-accent)"
                strokeWidth="10"
                opacity="0.25"
              />
              <circle
                cx="50"
                cy="50"
                r="40"
                fill="none"
                stroke="var(--color-xp)"
                strokeWidth="10"
                strokeLinecap="round"
                strokeDasharray={circumference}
                strokeDashoffset={circumference * (1 - pct / 100)}
                transform="rotate(-90 50 50)"
              />
            </svg>
            <div
              style={{
                position: 'absolute',
                inset: 0,
                display: 'grid',
                placeItems: 'center',
                textAlign: 'center',
              }}
            >
              <div>
                <p style={{ margin: 0, fontSize: '1.4rem', fontWeight: 700, lineHeight: 1 }}>
                  Level {profile.level}
                </p>
                <span style={{ fontSize: '0.7rem', color: 'var(--color-muted)' }}>current</span>
              </div>
            </div>
          </div>
          <div className="grow" style={{ minWidth: 0 }}>
            <h2 style={{ margin: '0 0 0.25rem', fontSize: '1.125rem' }}>Level progress</h2>
            <Progress value={pct} aria-label="XP progress to next level" />
            <p className="small" style={{ marginTop: '0.375rem' }}>
              {profile.xpIntoLevel} of {profile.xpForNext} XP to level {profile.level + 1}
            </p>
          </div>
        </div>
        <div
          style={{
            display: 'flex',
            flexWrap: 'wrap',
            gap: '1.5rem',
            marginTop: '1rem',
            color: 'var(--color-muted)',
            fontSize: '0.8125rem',
          }}
        >
          <span>
            Total XP: <b style={{ color: 'var(--color-foreground)' }}>{profile.totalXp}</b>
          </span>
          <span>
            Best streak today: <b style={{ color: 'var(--color-foreground)' }}>{best}</b> day
            {best === 1 ? '' : 's'}
          </span>
          <span>
            Open tasks: <b style={{ color: 'var(--color-foreground)' }}>{openTasks.length}</b>
          </span>
        </div>
      </div>

      <div className="cols" style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '1.25rem' }}>
        <section className="card" aria-label="Today's tasks">
          <h2>Today&apos;s tasks</h2>
          {openTasks.length === 0 ? (
            <p style={{ color: 'var(--color-muted)' }}>Nothing left to do. Nice work.</p>
          ) : (
            <ul style={{ listStyle: 'none', margin: 0, padding: 0 }}>
              {openTasks.slice(0, 8).map((task) => (
                <TodayTaskRow key={task.id} task={task} />
              ))}
            </ul>
          )}
        </section>

        <section className="card" aria-label="Today's habits">
          <h2>Habits</h2>
          {habits.length === 0 ? (
            <p style={{ color: 'var(--color-muted)' }}>No habits yet. Add one from the Habits screen.</p>
          ) : (
            <ul style={{ listStyle: 'none', margin: 0, padding: 0 }}>
              {habits.map((habit) => (
                <TodayHabitRow key={habit.id} habit={habit} />
              ))}
            </ul>
          )}
        </section>
      </div>
    </div>
  )
}

function bestStreak(habits: HabitView[]): number {
  return habits.reduce((max, h) => Math.max(max, h.currentStreak), 0)
}

function TodayTaskRow({ task }: { task: Task }) {
  const overdue = task.dueAt !== null && task.dueAt.slice(0, 10) < localToday()
  return (
    <li className="row" data-task-row={task.id}>
      <div className="grow">
        <b>{task.title}</b>
        {task.dueAt !== null && (
          <span className="small">
            {overdue ? 'Overdue · ' : 'Due '}
            {task.dueAt.slice(0, 10)}
          </span>
        )}
      </div>
      <span className={`tag ${task.priority}`}>{task.priority}</span>
    </li>
  )
}

function TodayHabitRow({ habit }: { habit: HabitView }) {
  return (
    <li className="row">
      <Sparkles aria-hidden className="text-xp" />
      <div className="grow">
        <b>{habit.title}</b>
        <span className="small">
          {habit.completedToday
            ? 'Checked in today'
            : `Checked in ${habit.currentStreak} day streak${habit.currentStreak === 1 ? '' : 's'}`}
        </span>
      </div>
      {habit.completedToday ? (
        <span className="tag" style={{ background: 'var(--color-ok)', color: '#fff' }}>
          Done
        </span>
      ) : (
        <span className="small">Not yet</span>
      )}
    </li>
  )
}
