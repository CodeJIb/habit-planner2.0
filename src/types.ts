// These mirror the Rust structs in src-tauri/src/models (serialized as camelCase).

export type Priority = 'low' | 'medium' | 'high' | 'urgent'
export type TaskStatus = 'todo' | 'in_progress' | 'done' | 'archived'
export type Frequency = 'daily' | 'weekly' | 'custom'

export interface Task {
  id: number
  title: string
  description: string | null
  status: TaskStatus
  priority: Priority
  categoryId: number | null
  dueAt: string | null
  scheduledStartAt: string | null
  estimatedDurationMinutes: number | null
  recurrenceRule: string | null
  createdAt: string
  updatedAt: string
  completedAt: string | null
  archivedAt: string | null
}

export interface TaskInput {
  title: string
  description?: string | null
  priority: Priority
  categoryId?: number | null
  dueAt?: string | null
  scheduledStartAt?: string | null
  estimatedDurationMinutes?: number | null
  recurrenceRule?: string | null
}

export interface Habit {
  id: number
  title: string
  description: string | null
  categoryId: number | null
  frequencyType: Frequency
  frequencyConfig: string | null
  targetCount: number | null
  reminderTime: string | null
  isActive: boolean
  createdAt: string
  archivedAt: string | null
}

export interface HabitView extends Habit {
  completedToday: boolean
  currentStreak: number
}

export interface HabitInput {
  title: string
  description?: string | null
  categoryId?: number | null
  frequencyType: Frequency
  frequencyConfig?: string | null
  targetCount?: number | null
  reminderTime?: string | null
}

export interface Category {
  id: number
  name: string
  color: string
  createdAt: string
}

export interface PlayerProfile {
  level: number
  totalXp: number
  xpIntoLevel: number
  xpForNext: number
}

export interface XpOutcome {
  xpDelta: number
  leveledUp: boolean
  profile: PlayerProfile
}

export interface TaskStatusResult {
  task: Task
  xp: XpOutcome
}

export interface HabitToggleResult {
  completed: boolean
  currentStreak: number
  xp: XpOutcome
}

export type QuestType = 'daily' | 'weekly'
export type QuestStatus = 'active' | 'completed' | 'expired'

export interface Quest {
  id: number
  code: string | null
  title: string
  description: string | null
  questType: QuestType
  metric: string
  targetValue: number
  xpReward: number
  startDate: string
  endDate: string
  status: QuestStatus
  periodKey: string | null
  completedAt: string | null
  xpRewarded: boolean
  createdAt: string
  currentValue: number
  completed: boolean
}

export interface Achievement {
  id: number
  code: string
  name: string
  description: string | null
  icon: string | null
  xpReward: number
  unlocked: boolean
  unlockedAt: string | null
}

export type GoalStatus = 'active' | 'completed' | 'archived'

export interface Goal {
  id: number
  title: string
  description: string | null
  targetDate: string | null
  status: GoalStatus
  createdAt: string
  completedAt: string | null
}

export interface GoalInput {
  title: string
  description?: string | null
  targetDate?: string | null
}

export interface GoalMilestone {
  id: number
  goalId: number
  title: string
  targetDate: string | null
  status: GoalStatus
  completedAt: string | null
}

export interface MilestoneInput {
  title: string
  targetDate?: string | null
}

export interface GoalView extends Goal {
  milestones: GoalMilestone[]
}

export interface GoalStatusResult {
  goal: GoalView
  xp: XpOutcome
}

export type BlockStatus = 'scheduled' | 'running' | 'done' | 'cancelled'
export type BlockType = 'task' | 'break' | 'other'

/** A calendar time block. `startAt`/`endAt` are local "YYYY-MM-DDTHH:MM". */
export interface ScheduledBlock {
  id: number
  taskId: number | null
  title: string
  startAt: string
  endAt: string
  blockType: BlockType
  status: BlockStatus
  createdAt: string
}

export interface ScheduledBlockInput {
  title: string
  startAt: string
  endAt: string
  taskId?: number | null
  blockType?: BlockType
}

export type FocusStatus = 'active' | 'completed' | 'cancelled'

/** One logged focus session (a Pomodoro). Timestamps are local "YYYY-MM-DDTHH:MM:SS". */
export interface FocusSession {
  id: number
  taskId: number | null
  startedAt: string
  endedAt: string | null
  plannedDurationMinutes: number
  actualDurationSeconds: number | null
  status: FocusStatus
}

export interface FocusSessionInput {
  taskId?: number | null
  plannedDurationMinutes: number
}

export interface FocusResult {
  session: FocusSession
  xp: XpOutcome
}

export interface FocusStats {
  todaySessions: number
  todayMinutes: number
  weekSessions: number
  weekMinutes: number
}

export interface AiConversation {
  id: number
  title: string | null
  createdAt: string
  updatedAt: string
}

export type AiRole = 'user' | 'assistant'

/** One persisted chat line. Tool traffic is never stored. */
export interface AiMessage {
  id: number
  conversationId: number
  role: AiRole
  content: string
  createdAt: string
}

export interface ChatInput {
  conversationId?: number | null
  message: string
}

/** Result of one chat turn: the thread, the new lines and any XP earned. */
export interface ChatResult {
  conversation: AiConversation
  messages: AiMessage[]
  xp: XpOutcome
}

/** The API key is never sent back to the UI, only whether one is stored. */
export interface AiSettings {
  apiKeySet: boolean
  /** Empty uses the built-in Anthropic provider; otherwise an OpenAI-compatible base URL. */
  baseUrl: string
  model: string
}

export interface AiSettingsInput {
  /** null/omitted leaves the stored key unchanged; empty string clears it. */
  apiKey?: string | null
  /** null/omitted leaves the base URL unchanged; empty string returns to Anthropic. */
  baseUrl?: string | null
  /** null/omitted leaves the model unchanged; empty string resets the default. */
  model?: string | null
}

/** Phone reminders, delivered through a free ntfy.sh topic. */
export interface NotificationSettings {
  enabled: boolean
  /** The topic the phone subscribes to. Empty means "not set up yet". */
  topic: string
  /** Empty means the public https://ntfy.sh server. */
  server: string
}

export interface NotificationSettingsInput {
  enabled?: boolean
  /** Omitted leaves it unchanged; empty string clears it. */
  topic?: string
  /** Omitted leaves it unchanged; empty string clears it. */
  server?: string
}

/** How many alerts a manual check pushed. */
export interface ReminderResult {
  sent: number
}

export type TtsSource = 'cloud' | 'system'

/** One voice from the Edge neural catalogue, for the cloud voice picker. */
export interface CloudVoice {
  /** Short name sent back for synthesis, e.g. "en-GB-RyanNeural". */
  id: string
  /** Dropdown text, e.g. "Ryan (Male, en-GB)". */
  label: string
  /** BCP-47 locale, used to sort British voices to the top. */
  locale: string
}

/** JARVIS text-to-speech settings, persisted in `app_settings`. */
export interface TtsSettings {
  enabled: boolean
  /** Where speech is produced: "cloud" (Edge neural voices) or "system" (offline). */
  source: TtsSource
  /** Voice name as reported by the OS. Empty means "auto-pick a voice". */
  voice: string
  /** Edge voice short name (e.g. "en-GB-RyanNeural") when source is "cloud". */
  cloudVoice: string
  /** 1.0 is normal speed; clamped to 0.5–2.0. */
  rate: number
  /** 1.0 is normal pitch; clamped to 0.5–2.0. System voice only. */
  pitch: number
}

export interface TtsSettingsInput {
  enabled?: boolean
  source?: TtsSource
  /** Omitted leaves it unchanged; empty string means "auto-pick". */
  voice?: string
  /** Omitted leaves it unchanged. */
  cloudVoice?: string
  rate?: number
  pitch?: number
}

/** One day on the XP-over-time chart: earned that day and the running total after it. */
export interface XpPoint {
  date: string
  amount: number
  cumulative: number
}

export interface DayCount {
  date: string
  count: number
}

export interface DayMinutes {
  date: string
  minutes: number
}

export interface AnalyticsSummary {
  totalXp: number
  level: number
  longestStreak: number
  tasksCompleted: number
  focusMinutes: number
  goalsCompleted: number
}

/** Everything the Analytics screen shows, fetched together when it opens. */
export interface Analytics {
  xp: XpPoint[]
  taskCompletions: DayCount[]
  focusMinutes: DayMinutes[]
  habitCheckins: string[]
  summary: AnalyticsSummary
}
