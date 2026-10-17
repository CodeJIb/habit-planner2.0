import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
/** True when running inside the Tauri window (false in a plain browser tab). */
export const isTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
/** Tauri rejects with the Rust error message (a string); normalise to a real Error. */
async function call(command, args) {
    if (!isTauri) {
        throw new Error('Not running inside Tauri. Start the app with: npm run tauri dev');
    }
    try {
        return await invoke(command, args);
    }
    catch (e) {
        throw new Error(typeof e === 'string' ? e : e instanceof Error ? e.message : 'Unknown error');
    }
}
/** Event name the Rust side uses to push streamed assistant text. */
export const AI_DELTA_EVENT = 'ai:delta';
/**
 * Subscribe to streamed assistant text. Resolves to an unsubscribe function.
 * A no-op outside Tauri so a plain browser tab does not throw.
 */
export function onAiDelta(handler) {
    if (!isTauri)
        return Promise.resolve(() => { });
    return listen(AI_DELTA_EVENT, (event) => handler(event.payload));
}
export const api = {
    // tasks
    listTasks: (status) => call('list_tasks', { status: status ?? null }),
    createTask: (input) => call('create_task', { input }),
    updateTask: (id, input) => call('update_task', { id, input }),
    setTaskStatus: (id, status) => call('set_task_status', { id, status }),
    deleteTask: (id) => call('delete_task', { id }),
    // habits
    listHabits: (today) => call('list_habits', { today }),
    createHabit: (input) => call('create_habit', { input }),
    deleteHabit: (id) => call('delete_habit', { id }),
    toggleHabit: (habitId, localDate) => call('toggle_habit_completion', { habitId, localDate }),
    // categories + profile
    listCategories: () => call('list_categories'),
    getProfile: () => call('get_player_profile'),
    // quests + achievements
    listQuests: () => call('list_quests'),
    listAchievements: () => call('list_achievements'),
    // goals + milestones
    listGoals: (status) => call('list_goals', { status: status ?? null }),
    createGoal: (input) => call('create_goal', { input }),
    updateGoal: (id, input) => call('update_goal', { id, input }),
    setGoalStatus: (id, status) => call('set_goal_status', { id, status }),
    deleteGoal: (id) => call('delete_goal', { id }),
    addGoalMilestone: (goalId, input) => call('add_goal_milestone', { goalId, input }),
    setGoalMilestoneStatus: (id, status) => call('set_goal_milestone_status', { id, status }),
    deleteGoalMilestone: (id) => call('delete_goal_milestone', { id }),
    // calendar (scheduled blocks)
    listBlocks: (startDate, endDate) => call('list_blocks', {
        startDate: startDate ?? null,
        endDate: endDate ?? null,
    }),
    createBlock: (input) => call('create_block', { input }),
    updateBlock: (id, input) => call('update_block', { id, input }),
    setBlockStatus: (id, status) => call('set_block_status', { id, status }),
    deleteBlock: (id) => call('delete_block', { id }),
    // focus sessions
    listFocusSessions: (taskId, limit) => call('list_focus_sessions', {
        taskId: taskId ?? null,
        limit: limit ?? null,
    }),
    startFocusSession: (input) => call('start_focus_session', { input }),
    completeFocusSession: (id, actualDurationSeconds) => call('complete_focus_session', { id, actualDurationSeconds }),
    cancelFocusSession: (id) => call('cancel_focus_session', { id }),
    deleteFocusSession: (id) => call('delete_focus_session', { id }),
    focusStats: () => call('focus_stats'),
    // assistant (JARVIS)
    chat: (request) => call('ai_chat', { request }),
    /** Same as `chat`, but the assistant's words stream in via `onAiDelta`. */
    chatStream: (request) => call('ai_chat_stream', { request }),
    listConversations: () => call('list_conversations'),
    getConversation: (id) => call('get_conversation', { id }),
    deleteConversation: (id) => call('delete_conversation', { id }),
    getAiSettings: () => call('get_ai_settings'),
    setAiSettings: (input) => call('set_ai_settings', { input }),
};
