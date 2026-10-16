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
