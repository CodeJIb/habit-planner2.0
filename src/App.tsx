import { CalendarDays, CheckCheck, LayoutGrid, Moon, Sparkles, Sun, Target, Timer, TrendingUp, Users, X } from 'lucide-react'
import { useCallback, useEffect, useState } from 'react'
import { TodayPage } from './components/TodayPage'
import { TasksPage } from './components/TasksPage'
import { HabitsPage } from './components/HabitsPage'
import { PlaceholderPage } from './components/PlaceholderPage'
import { JarvisPanel } from './components/JarvisPanel'
import { api } from './lib/api'
import { localToday } from './lib/dates'
import type { HabitInput, HabitView, PlayerProfile, Task, TaskInput } from './types'

type View = 'today' | 'tasks' | 'habits' | 'quests' | 'goals' | 'calendar' | 'focus' | 'analytics'

const NAV_ITEMS: { id: View; label: string; icon: React.ReactNode; soon?: boolean }[] = [
  { id: 'today', label: 'Today', icon: <Sparkles aria-hidden /> },
  { id: 'tasks', label: 'Tasks', icon: <CheckCheck aria-hidden /> },
  { id: 'habits', label: 'Habits', icon: <Users aria-hidden /> },
  { id: 'quests', label: 'Quests', icon: <LayoutGrid aria-hidden />, soon: true },
  { id: 'goals', label: 'Goals', icon: <Target aria-hidden />, soon: true },
  { id: 'calendar', label: 'Calendar', icon: <CalendarDays aria-hidden />, soon: true },
  { id: 'focus', label: 'Focus', icon: <Timer aria-hidden />, soon: true },
  { id: 'analytics', label: 'Analytics', icon: <TrendingUp aria-hidden />, soon: true },
]

const COMING_SOON: Partial<Record<View, { title: string; body: string }>> = {
  quests: {
    title: 'Quests',
    body: 'Daily quests, weekly challenges and milestone rewards arrive with gamification (Phase 3).',
  },
  goals: {
    title: 'Goals',
    body: 'Goals and milestones are part of Phase 4.',
  },
  calendar: {
    title: 'Calendar',
    body: 'The calendar view is part of Phase 4.',
  },
  focus: {
    title: 'Focus',
    body: 'Focus mode and the Pomodoro timer are part of Phase 4.',
  },
  analytics: {
    title: 'Analytics',
    body: 'Charts, reports and statistics arrive with Phase 7.',
  },
}

function isDark(): boolean {
  return document.documentElement.classList.contains('dark')
}

export default function App() {
  const [view, setView] = useState<View>('today')
  const [tasks, setTasks] = useState<Task[]>([])
  const [habits, setHabits] = useState<HabitView[]>([])
  const [profile, setProfile] = useState<PlayerProfile | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [notice, setNotice] = useState<string | null>(null)
  const [assistantMessages] = useState<string[]>([])
  const [collapsed, setCollapsed] = useState(false)

  // Run an async action; show any Rust error in the banner instead of failing silently.
  const run = useCallback(async (action: () => Promise<void>) => {
    try {
      setError(null)
      await action()
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e))
    }
  }, [])

  const load = useCallback(
    () =>
      run(async () => {
        const [t, h, p] = await Promise.all([
          api.listTasks(),
          api.listHabits(localToday()),
          api.getProfile(),
        ])
        setTasks(t)
        setHabits(h)
        setProfile(p)
      }),
    [run],
  )

  useEffect(() => {
    void load()
  }, [load])

  // Auto-hide the level-up / XP notice.
  useEffect(() => {
    if (!notice) return
    const id = window.setTimeout(() => setNotice(null), 3500)
    return () => window.clearTimeout(id)
  }, [notice])

  function applyXp(outcome: { xpDelta: number; leveledUp: boolean; profile: PlayerProfile }) {
    setProfile(outcome.profile)
    if (outcome.leveledUp) {
      setNotice(`Level up! You reached level ${outcome.profile.level}.`)
    } else if (outcome.xpDelta > 0) {
      setNotice(`+${outcome.xpDelta} XP`)
    }
  }

  const createTask = (input: TaskInput) =>
    run(async () => {
      const task = await api.createTask(input)
      setTasks((prev) => [task, ...prev])
    })

  const toggleTask = (task: Task) =>
    run(async () => {
      const result = await api.setTaskStatus(task.id, task.status === 'done' ? 'todo' : 'done')
      setTasks((prev) => prev.map((t) => (t.id === result.task.id ? result.task : t)))
      applyXp(result.xp)
    })

  const deleteTask = (task: Task) =>
    run(async () => {
      await api.deleteTask(task.id)
      setTasks((prev) => prev.filter((t) => t.id !== task.id))
    })

  const createHabit = (input: HabitInput) =>
    run(async () => {
      await api.createHabit(input)
      setHabits(await api.listHabits(localToday()))
    })

  const toggleHabit = (habit: HabitView) =>
    run(async () => {
      const result = await api.toggleHabit(habit.id, localToday())
      setHabits((prev) =>
        prev.map((h) =>
          h.id === habit.id
            ? {
                ...h,
                completedToday: result.completed,
                currentStreak: result.currentStreak,
              }
            : h,
        ),
      )
      applyXp(result.xp)
    })

  const deleteHabit = (habit: HabitView) =>
    run(async () => {
      await api.deleteHabit(habit.id)
      setHabits((prev) => prev.filter((h) => h.id !== habit.id))
    })

  function toggleTheme() {
    document.documentElement.classList.toggle('dark', isDark())
  }

  return (
    <div className="app">
      <aside className="sidebar" style={collapsed ? { display: 'none' } : undefined}>
        <div className="brand">Habit Planner</div>
        <nav aria-label="Main">
          {NAV_ITEMS.map((item) => (
            <button
              key={item.id}
              type="button"
              aria-current={view === item.id ? 'page' : undefined}
              onClick={() => setView(item.id)}
            >
              {item.icon}
              <span>{item.label}</span>
              {item.soon && <span data-soon>soon</span>}
            </button>
          ))}
        </nav>
        <div className="spacer" />
        <button type="button" onClick={toggleTheme} aria-label="Switch theme">
          {isDark() ? <Sun aria-hidden /> : <Moon aria-hidden />}
          <span>{isDark() ? 'Light theme' : 'Dark theme'}</span>
        </button>
        <button
          type="button"
          onClick={() => setCollapsed(!collapsed)}
          aria-expanded={!collapsed}
          aria-label={collapsed ? 'Expand sidebar' : 'Collapse sidebar'}
        >
          {collapsed ? 'Expand' : 'Collapse'}
        </button>
      </aside>

      <main className="main">
        {view === 'today' && (
          <TodayPage tasks={tasks} habits={habits} profile={profile ?? FALLBACK_PROFILE} />
        )}
        {view === 'tasks' && (
          <TasksPage
            tasks={tasks}
            onCreate={createTask}
            onToggle={toggleTask}
            onDelete={deleteTask}
          />
        )}
        {view === 'habits' && (
          <HabitsPage
            habits={habits}
            onCreate={createHabit}
            onToggle={toggleHabit}
            onDelete={deleteHabit}
          />
        )}
        {COMING_SOON[view] && (
          <PlaceholderPage
            title={COMING_SOON[view]!.title}
            body={COMING_SOON[view]!.body}
          />
        )}
      </main>

      <JarvisPanel messages={assistantMessages} />

      {error && (
        <div
          role="alert"
          style={{
            position: 'fixed',
            top: '1rem',
            right: '1rem',
            zIndex: 60,
            display: 'flex',
            gap: '0.75rem',
            alignItems: 'flex-start',
            maxWidth: '26rem',
            border: '1px solid var(--color-destructive)',
            borderRadius: 'var(--radius-md)',
            background: 'var(--color-card)',
            padding: '0.75rem 1rem',
            fontSize: '0.875rem',
          }}
        >
          <span>{error}</span>
          <button type="button" onClick={() => setError(null)} aria-label="Dismiss error">
            <X aria-hidden />
          </button>
        </div>
      )}

      {notice && (
        <div className="toast show" role="status">
          {notice}
        </div>
      )}
    </div>
  )
}

const FALLBACK_PROFILE = {
  level: 1,
  totalXp: 0,
  xpIntoLevel: 0,
  xpForNext: 100,
}
