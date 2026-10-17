import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import type {
  Achievement,
  AiConversation,
  AiMessage,
  AiSettings,
  AiSettingsInput,
  BlockStatus,
  Category,
  ChatInput,
  ChatResult,
  FocusResult,
  FocusSession,
  FocusSessionInput,
  FocusStats,
  GoalInput,
  GoalStatus,
  GoalStatusResult,
  GoalView,
  Habit,
  HabitInput,
  HabitToggleResult,
  HabitView,
  MilestoneInput,
  PlayerProfile,
  Quest,
  ScheduledBlock,
  ScheduledBlockInput,
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

/** Event name the Rust side uses to push streamed assistant text. */
export const AI_DELTA_EVENT = 'ai:delta'

/**
 * Subscribe to streamed assistant text. Resolves to an unsubscribe function.
 * A no-op outside Tauri so a plain browser tab does not throw.
 */
export function onAiDelta(handler: (delta: string) => void): Promise<() => void> {
  if (!isTauri) return Promise.resolve(() => {})
  return listen<string>(AI_DELTA_EVENT, (event) => handler(event.payload))
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

  // quests + achievements
  listQuests: () => call<Quest[]>('list_quests'),
  listAchievements: () => call<Achievement[]>('list_achievements'),

  // goals + milestones
  listGoals: (status?: GoalStatus) => call<GoalView[]>('list_goals', { status: status ?? null }),
  createGoal: (input: GoalInput) => call<GoalView>('create_goal', { input }),
  updateGoal: (id: number, input: GoalInput) => call<GoalView>('update_goal', { id, input }),
  setGoalStatus: (id: number, status: GoalStatus) =>
    call<GoalStatusResult>('set_goal_status', { id, status }),
  deleteGoal: (id: number) => call<void>('delete_goal', { id }),
  addGoalMilestone: (goalId: number, input: MilestoneInput) =>
    call<GoalView>('add_goal_milestone', { goalId, input }),
  setGoalMilestoneStatus: (id: number, status: GoalStatus) =>
    call<GoalView>('set_goal_milestone_status', { id, status }),
  deleteGoalMilestone: (id: number) => call<GoalView>('delete_goal_milestone', { id }),

  // calendar (scheduled blocks)
  listBlocks: (startDate?: string, endDate?: string) =>
    call<ScheduledBlock[]>('list_blocks', {
      startDate: startDate ?? null,
      endDate: endDate ?? null,
    }),
  createBlock: (input: ScheduledBlockInput) => call<ScheduledBlock>('create_block', { input }),
  updateBlock: (id: number, input: ScheduledBlockInput) =>
    call<ScheduledBlock>('update_block', { id, input }),
  setBlockStatus: (id: number, status: BlockStatus) =>
    call<ScheduledBlock>('set_block_status', { id, status }),
  deleteBlock: (id: number) => call<void>('delete_block', { id }),

  // focus sessions
  listFocusSessions: (taskId?: number, limit?: number) =>
    call<FocusSession[]>('list_focus_sessions', {
      taskId: taskId ?? null,
      limit: limit ?? null,
    }),
  startFocusSession: (input: FocusSessionInput) =>
    call<FocusSession>('start_focus_session', { input }),
  completeFocusSession: (id: number, actualDurationSeconds: number) =>
    call<FocusResult>('complete_focus_session', { id, actualDurationSeconds }),
  cancelFocusSession: (id: number) => call<FocusResult>('cancel_focus_session', { id }),
  deleteFocusSession: (id: number) => call<void>('delete_focus_session', { id }),
  focusStats: () => call<FocusStats>('focus_stats'),

  // assistant (JARVIS)
  chat: (request: ChatInput) => call<ChatResult>('ai_chat', { request }),
  /** Same as `chat`, but the assistant's words stream in via `onAiDelta`. */
  chatStream: (request: ChatInput) => call<ChatResult>('ai_chat_stream', { request }),
  listConversations: () => call<AiConversation[]>('list_conversations'),
  getConversation: (id: number) => call<AiMessage[]>('get_conversation', { id }),
  deleteConversation: (id: number) => call<void>('delete_conversation', { id }),
  getAiSettings: () => call<AiSettings>('get_ai_settings'),
  setAiSettings: (input: AiSettingsInput) => call<AiSettings>('set_ai_settings', { input }),
}
