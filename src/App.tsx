import { CalendarDays, CheckCheck, LayoutGrid, Moon, Sparkles, Sun, Target, Timer, TrendingUp, Users, X } from 'lucide-react'
import { Suspense, lazy, useCallback, useEffect, useState } from 'react'
import { TodayPage } from './components/TodayPage'
import { TasksPage } from './components/TasksPage'
import { HabitsPage } from './components/HabitsPage'
import { QuestsPage } from './components/QuestsPage'
import { GoalsPage } from './components/GoalsPage'
import { CalendarPage } from './components/CalendarPage'
import { FocusPage } from './components/FocusPage'
import { PlaceholderPage } from './components/PlaceholderPage'
import { api, onAiDelta } from './lib/api'
import { localToday } from './lib/dates'
import type {
  Achievement,
  AiConversation,
  AiMessage,
  AiSettings,
  AiSettingsInput,
  BlockStatus,
  FocusSession,
  FocusSessionInput,
  FocusStats,
  GoalInput,
  GoalMilestone,
  GoalStatus,
  GoalView,
  HabitInput,
  HabitView,
  MilestoneInput,
  PlayerProfile,
  Quest,
  ScheduledBlock,
  ScheduledBlockInput,
  Task,
  TaskInput,
} from './types'

type View = 'today' | 'tasks' | 'habits' | 'quests' | 'goals' | 'calendar' | 'focus' | 'analytics'

// The assistant panel is split into its own chunk so the initial screen paints
// without loading the chat UI (matters on low-end phones).
const JarvisPanel = lazy(() =>
  import('./components/JarvisPanel').then((module) => ({ default: module.JarvisPanel })),
)

const NAV_ITEMS: { id: View; label: string; icon: React.ReactNode; soon?: boolean }[] = [
  { id: 'today', label: 'Today', icon: <Sparkles aria-hidden /> },
  { id: 'tasks', label: 'Tasks', icon: <CheckCheck aria-hidden /> },
  { id: 'habits', label: 'Habits', icon: <Users aria-hidden /> },
  { id: 'quests', label: 'Quests', icon: <LayoutGrid aria-hidden /> },
  { id: 'goals', label: 'Goals', icon: <Target aria-hidden /> },
  { id: 'calendar', label: 'Calendar', icon: <CalendarDays aria-hidden /> },
  { id: 'focus', label: 'Focus', icon: <Timer aria-hidden /> },
  { id: 'analytics', label: 'Analytics', icon: <TrendingUp aria-hidden />, soon: true },
]

const COMING_SOON: Partial<Record<View, { title: string; body: string }>> = {
  analytics: {
    title: 'Analytics',
    body: 'Charts, reports and statistics arrive with Phase 7.',
  },
}

/** Initial theme: an explicit choice if one was saved, otherwise the OS setting. */
function initialDark(): boolean {
  const saved = localStorage.getItem('theme')
  if (saved === 'dark') return true
  if (saved === 'light') return false
  return window.matchMedia('(prefers-color-scheme: dark)').matches
}

export default function App() {
  const [view, setView] = useState<View>('today')
  const [tasks, setTasks] = useState<Task[]>([])
  const [habits, setHabits] = useState<HabitView[]>([])
  const [quests, setQuests] = useState<Quest[]>([])
  const [achievements, setAchievements] = useState<Achievement[]>([])
  const [goals, setGoals] = useState<GoalView[]>([])
  const [blocks, setBlocks] = useState<ScheduledBlock[]>([])
  const [focusSessions, setFocusSessions] = useState<FocusSession[]>([])
  const [focusStats, setFocusStats] = useState<FocusStats | null>(null)
  const [profile, setProfile] = useState<PlayerProfile | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [notice, setNotice] = useState<string | null>(null)
  const [assistantMessages, setAssistantMessages] = useState<AiMessage[]>([])
  const [conversations, setConversations] = useState<AiConversation[]>([])
  const [activeConversation, setActiveConversation] = useState<number | null>(null)
  const [assistantBusy, setAssistantBusy] = useState(false)
  /** Assistant text streaming in for the in-flight turn. */
  const [assistantStream, setAssistantStream] = useState('')
  /** The user's line, shown immediately while the reply is on the way. */
  const [pendingUser, setPendingUser] = useState<string | null>(null)
  const [aiSettings, setAiSettings] = useState<AiSettings | null>(null)
  const [collapsed, setCollapsed] = useState(false)
  const [dark, setDark] = useState(initialDark)

  // Run an async action; show any Rust error in the banner instead of failing silently.
  const run = useCallback(async (action: () => Promise<void>) => {
    try {
      setError(null)
      await action()
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e))
    }
  }, [])

  // Quest and achievement XP are awarded in Rust, so reload them together with the
  // profile afterwards to reflect any completions/unlocks they produced. These calls
  // are sequential because each may write (award XP) inside its own transaction.
  const refreshQuests = useCallback(async () => {
    setQuests(await api.listQuests())
    setAchievements(await api.listAchievements())
  }, [])

  const load = useCallback(
    () =>
      run(async () => {
        const [t, h, g, b, f, fs, settings] = await Promise.all([
          api.listTasks(),
          api.listHabits(localToday()),
          api.listGoals(),
          api.listBlocks(),
          api.listFocusSessions(),
          api.focusStats(),
          api.getAiSettings(),
        ])
        setTasks(t)
        setHabits(h)
        setGoals(g)
        setBlocks(b)
        setFocusSessions(f)
        setFocusStats(fs)
        setAiSettings(settings)
        await refreshQuests()
        // Profile last: listing quests/achievements can award XP as a side effect.
        setProfile(await api.getProfile())
      }),
    [run, refreshQuests],
  )

  // Load the assistant threads once, separately from the dashboard data.
  const loadConversations = useCallback(async () => {
    setConversations(await api.listConversations())
  }, [])

  // Load initial data from the backend. The synchronous setState lives in `run`,
  // which is why the next line is flagged; clearing the error on mount is a no-op.
  useEffect(() => {
    // oxlint-disable-next-line react/set-state-in-effect
    void load()
  }, [load])

  useEffect(() => {
    // oxlint-disable-next-line react/set-state-in-effect
    void loadConversations()
  }, [loadConversations])

  // Stream assistant text into the panel as it arrives.
  useEffect(() => {
    let unlisten: (() => void) | undefined
    let cancelled = false
    void onAiDelta((delta) => setAssistantStream((prev) => prev + delta)).then((stop) => {
      if (cancelled) stop()
      else unlisten = stop
    })
    return () => {
      cancelled = true
      unlisten?.()
    }
  }, [])

  // Keep the <html> class and the saved preference in sync with the chosen theme.
  useEffect(() => {
    document.documentElement.classList.toggle('dark', dark)
    localStorage.setItem('theme', dark ? 'dark' : 'light')
  }, [dark])

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
      await refreshQuests()
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
      await refreshQuests()
    })

  const deleteHabit = (habit: HabitView) =>
    run(async () => {
      await api.deleteHabit(habit.id)
      setHabits((prev) => prev.filter((h) => h.id !== habit.id))
    })

  const createGoal = (input: GoalInput) =>
    run(async () => {
      const goal = await api.createGoal(input)
      setGoals((prev) => [goal, ...prev])
    })

  const setGoalStatus = (goal: GoalView, status: GoalStatus) =>
    run(async () => {
      const result = await api.setGoalStatus(goal.id, status)
      setGoals((prev) => prev.map((g) => (g.id === result.goal.id ? result.goal : g)))
      applyXp(result.xp)
    })

  const deleteGoal = (goal: GoalView) =>
    run(async () => {
      await api.deleteGoal(goal.id)
      setGoals((prev) => prev.filter((g) => g.id !== goal.id))
    })

  const addGoalMilestone = (goal: GoalView, input: MilestoneInput) =>
    run(async () => {
      const updated = await api.addGoalMilestone(goal.id, input)
      setGoals((prev) => prev.map((g) => (g.id === updated.id ? updated : g)))
    })

  const toggleGoalMilestone = (milestone: GoalMilestone) =>
    run(async () => {
      const status: GoalStatus = milestone.status === 'completed' ? 'active' : 'completed'
      const updated = await api.setGoalMilestoneStatus(milestone.id, status)
      setGoals((prev) => prev.map((g) => (g.id === updated.id ? updated : g)))
    })

  const deleteGoalMilestone = (milestone: GoalMilestone) =>
    run(async () => {
      const updated = await api.deleteGoalMilestone(milestone.id)
      setGoals((prev) => prev.map((g) => (g.id === updated.id ? updated : g)))
    })

  const createBlock = (input: ScheduledBlockInput) =>
    run(async () => {
      const block = await api.createBlock(input)
      setBlocks((prev) => [...prev, block])
    })

  const setBlockStatus = (block: ScheduledBlock, status: BlockStatus) =>
    run(async () => {
      const updated = await api.setBlockStatus(block.id, status)
      setBlocks((prev) => prev.map((b) => (b.id === updated.id ? updated : b)))
    })

  const deleteBlock = (block: ScheduledBlock) =>
    run(async () => {
      await api.deleteBlock(block.id)
      setBlocks((prev) => prev.filter((b) => b.id !== block.id))
    })

  // Starting returns the new session so the timer can hold its id; errors surface in the banner.
  const startFocus = async (input: FocusSessionInput): Promise<FocusSession | null> => {
    try {
      setError(null)
      const session = await api.startFocusSession(input)
      setFocusSessions((prev) => [session, ...prev])
      return session
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e))
      return null
    }
  }

  const refreshFocusStats = async () => setFocusStats(await api.focusStats())

  const completeFocus = (id: number, actualDurationSeconds: number) =>
    run(async () => {
      const result = await api.completeFocusSession(id, actualDurationSeconds)
      setFocusSessions((prev) =>
        prev.map((s) => (s.id === result.session.id ? result.session : s)),
      )
      applyXp(result.xp)
      await refreshFocusStats()
    })

  const cancelFocus = (id: number) =>
    run(async () => {
      const result = await api.cancelFocusSession(id)
      setFocusSessions((prev) =>
        prev.map((s) => (s.id === result.session.id ? result.session : s)),
      )
      await refreshFocusStats()
    })

  const deleteFocus = (session: FocusSession) =>
    run(async () => {
      await api.deleteFocusSession(session.id)
      setFocusSessions((prev) => prev.filter((s) => s.id !== session.id))
      await refreshFocusStats()
      // Deleting a completed session refunds XP, so refresh the profile too.
      setProfile(await api.getProfile())
    })

  // Send a chat turn. Actions the assistant takes (create/complete) write to the
  // database, so anything it may have touched is reloaded afterwards.
  const sendAssistant = async (text: string) => {
    setAssistantBusy(true)
    setAssistantStream('')
    setPendingUser(text)
    try {
      setError(null)
      const result = await api.chatStream({ conversationId: activeConversation, message: text })
      setAssistantMessages((prev) => [...prev, ...result.messages])
      setActiveConversation(result.conversation.id)
      setConversations((prev) => [
        result.conversation,
        ...prev.filter((c) => c.id !== result.conversation.id),
      ])
      applyXp(result.xp)
      await load()
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e))
    } finally {
      setAssistantBusy(false)
      setAssistantStream('')
      setPendingUser(null)
    }
  }

  const selectConversation = (id: number) =>
    run(async () => {
      setAssistantMessages(await api.getConversation(id))
      setActiveConversation(id)
    })

  function newConversation() {
    setActiveConversation(null)
    setAssistantMessages([])
  }

  const deleteConversation = (id: number) =>
    run(async () => {
      await api.deleteConversation(id)
      setConversations((prev) => prev.filter((c) => c.id !== id))
      if (activeConversation === id) {
        setActiveConversation(null)
        setAssistantMessages([])
      }
    })

  const saveAiSettings = (input: AiSettingsInput) =>
    run(async () => {
      setAiSettings(await api.setAiSettings(input))
    })

  function toggleTheme() {
    setDark((value) => !value)
  }

  return (
    <div className="app" data-collapsed={collapsed}>
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
          {dark ? <Sun aria-hidden /> : <Moon aria-hidden />}
          <span>{dark ? 'Light theme' : 'Dark theme'}</span>
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
        {view === 'quests' && <QuestsPage quests={quests} achievements={achievements} />}
        {view === 'goals' && (
          <GoalsPage
            goals={goals}
            onCreate={createGoal}
            onSetStatus={setGoalStatus}
            onDelete={deleteGoal}
            onAddMilestone={addGoalMilestone}
            onToggleMilestone={toggleGoalMilestone}
            onDeleteMilestone={deleteGoalMilestone}
          />
        )}
        {view === 'calendar' && (
          <CalendarPage
            blocks={blocks}
            tasks={tasks}
            onCreate={createBlock}
            onSetStatus={setBlockStatus}
            onDelete={deleteBlock}
          />
        )}
        {view === 'focus' && (
          <FocusPage
            sessions={focusSessions}
            stats={focusStats}
            tasks={tasks}
            onStart={startFocus}
            onComplete={completeFocus}
            onCancel={cancelFocus}
            onDelete={deleteFocus}
          />
        )}
        {COMING_SOON[view] && (
          <PlaceholderPage
            title={COMING_SOON[view]!.title}
            body={COMING_SOON[view]!.body}
          />
        )}
      </main>

      <Suspense fallback={<aside className="assistant" aria-hidden />}>
        <JarvisPanel
          messages={assistantMessages}
          conversations={conversations}
          activeConversationId={activeConversation}
          busy={assistantBusy}
          streaming={assistantStream}
          pendingUser={pendingUser}
          settings={aiSettings}
          onSend={sendAssistant}
          onSelectConversation={selectConversation}
          onNewConversation={newConversation}
          onDeleteConversation={deleteConversation}
          onSaveSettings={saveAiSettings}
        />
      </Suspense>

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
