import { invoke } from '@tauri-apps/api/core'
import type {
  Category,
  Habit,
  HabitInput,
  HabitToggleResult,
  HabitView,
  PlayerProfile,
  Task,
  TaskInput,
  TaskStatus,
  TaskStatusResult,
} from '../types'

/** True when running inside the Tauri window (false in a plain browser tab). */
export const isTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window

/** Tauri rejects with the Rust error message (a string); normalise to a real Error. */
async function call<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  if (!isTauri) {
    throw new Error('Not running inside Tauri. Start the app with: npm run tauri dev')
  }
  try {
    return await invoke<T>(command, args)
  } catch (e) {
    throw new Error(typeof e === 'string' ? e : e instanceof Error ? e.message : 'Unknown error')
  }
}

export const api = {
  // tasks
  listTasks: (status?: TaskStatus) => call<Task[]>('list_tasks', { status: status ?? null }),
  createTask: (input: TaskInput) => call<Task>('create_task', { input }),
  updateTask: (id: number, input: TaskInput) => call<Task>('update_task', { id, input }),
  setTaskStatus: (id: number, status: TaskStatus) =>
    call<TaskStatusResult>('set_task_status', { id, status }),
  deleteTask: (id: number) => call<void>('delete_task', { id }),

  // habits
  listHabits: (today: string) => call<HabitView[]>('list_habits', { today }),
  createHabit: (input: HabitInput) => call<Habit>('create_habit', { input }),
  deleteHabit: (id: number) => call<void>('delete_habit', { id }),
  toggleHabit: (habitId: number, localDate: string) =>
    call<HabitToggleResult>('toggle_habit_completion', { habitId, localDate }),

  // categories + profile
  listCategories: () => call<Category[]>('list_categories'),
  getProfile: () => call<PlayerProfile>('get_player_profile'),
}
