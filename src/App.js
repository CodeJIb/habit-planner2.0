import { jsx as _jsx, jsxs as _jsxs } from "react/jsx-runtime";
import { CalendarDays, CheckCheck, LayoutGrid, Moon, Sparkles, Sun, Target, Timer, TrendingUp, Users, X } from 'lucide-react';
import { Suspense, lazy, useCallback, useEffect, useState } from 'react';
import { TodayPage } from './components/TodayPage';
import { TasksPage } from './components/TasksPage';
import { HabitsPage } from './components/HabitsPage';
import { QuestsPage } from './components/QuestsPage';
import { GoalsPage } from './components/GoalsPage';
import { CalendarPage } from './components/CalendarPage';
import { FocusPage } from './components/FocusPage';
import { PlaceholderPage } from './components/PlaceholderPage';
import { api, onAiDelta } from './lib/api';
import { localToday } from './lib/dates';
// The assistant panel is split into its own chunk so the initial screen paints
// without loading the chat UI (matters on low-end phones).
const JarvisPanel = lazy(() => import('./components/JarvisPanel').then((module) => ({ default: module.JarvisPanel })));
const NAV_ITEMS = [
    { id: 'today', label: 'Today', icon: _jsx(Sparkles, { "aria-hidden": true }) },
    { id: 'tasks', label: 'Tasks', icon: _jsx(CheckCheck, { "aria-hidden": true }) },
    { id: 'habits', label: 'Habits', icon: _jsx(Users, { "aria-hidden": true }) },
    { id: 'quests', label: 'Quests', icon: _jsx(LayoutGrid, { "aria-hidden": true }) },
    { id: 'goals', label: 'Goals', icon: _jsx(Target, { "aria-hidden": true }) },
    { id: 'calendar', label: 'Calendar', icon: _jsx(CalendarDays, { "aria-hidden": true }) },
    { id: 'focus', label: 'Focus', icon: _jsx(Timer, { "aria-hidden": true }) },
    { id: 'analytics', label: 'Analytics', icon: _jsx(TrendingUp, { "aria-hidden": true }), soon: true },
];
const COMING_SOON = {
    analytics: {
        title: 'Analytics',
        body: 'Charts, reports and statistics arrive with Phase 7.',
    },
};
/** Initial theme: an explicit choice if one was saved, otherwise the OS setting. */
function initialDark() {
    const saved = localStorage.getItem('theme');
    if (saved === 'dark')
        return true;
    if (saved === 'light')
        return false;
    return window.matchMedia('(prefers-color-scheme: dark)').matches;
}
export default function App() {
    const [view, setView] = useState('today');
    const [tasks, setTasks] = useState([]);
    const [habits, setHabits] = useState([]);
    const [quests, setQuests] = useState([]);
    const [achievements, setAchievements] = useState([]);
    const [goals, setGoals] = useState([]);
    const [blocks, setBlocks] = useState([]);
    const [focusSessions, setFocusSessions] = useState([]);
    const [focusStats, setFocusStats] = useState(null);
    const [profile, setProfile] = useState(null);
    const [error, setError] = useState(null);
    const [notice, setNotice] = useState(null);
    const [assistantMessages, setAssistantMessages] = useState([]);
    const [conversations, setConversations] = useState([]);
    const [activeConversation, setActiveConversation] = useState(null);
    const [assistantBusy, setAssistantBusy] = useState(false);
    /** Assistant text streaming in for the in-flight turn. */
    const [assistantStream, setAssistantStream] = useState('');
    /** The user's line, shown immediately while the reply is on the way. */
    const [pendingUser, setPendingUser] = useState(null);
    const [aiSettings, setAiSettings] = useState(null);
    const [collapsed, setCollapsed] = useState(false);
    const [dark, setDark] = useState(initialDark);
    // Run an async action; show any Rust error in the banner instead of failing silently.
    const run = useCallback(async (action) => {
        try {
            setError(null);
            await action();
        }
        catch (e) {
            setError(e instanceof Error ? e.message : String(e));
        }
    }, []);
    // Quest and achievement XP are awarded in Rust, so reload them together with the
    // profile afterwards to reflect any completions/unlocks they produced. These calls
    // are sequential because each may write (award XP) inside its own transaction.
    const refreshQuests = useCallback(async () => {
        setQuests(await api.listQuests());
        setAchievements(await api.listAchievements());
    }, []);
    const load = useCallback(() => run(async () => {
        const [t, h, g, b, f, fs, settings] = await Promise.all([
            api.listTasks(),
            api.listHabits(localToday()),
            api.listGoals(),
            api.listBlocks(),
            api.listFocusSessions(),
            api.focusStats(),
            api.getAiSettings(),
        ]);
        setTasks(t);
        setHabits(h);
        setGoals(g);
        setBlocks(b);
        setFocusSessions(f);
        setFocusStats(fs);
        setAiSettings(settings);
        await refreshQuests();
        // Profile last: listing quests/achievements can award XP as a side effect.
        setProfile(await api.getProfile());
    }), [run, refreshQuests]);
    // Load the assistant threads once, separately from the dashboard data.
    const loadConversations = useCallback(async () => {
        setConversations(await api.listConversations());
    }, []);
    // Load initial data from the backend. The synchronous setState lives in `run`,
    // which is why the next line is flagged; clearing the error on mount is a no-op.
    useEffect(() => {
        // oxlint-disable-next-line react/set-state-in-effect
        void load();
    }, [load]);
    useEffect(() => {
        // oxlint-disable-next-line react/set-state-in-effect
        void loadConversations();
    }, [loadConversations]);
    // Stream assistant text into the panel as it arrives.
    useEffect(() => {
        let unlisten;
        let cancelled = false;
        void onAiDelta((delta) => setAssistantStream((prev) => prev + delta)).then((stop) => {
            if (cancelled)
                stop();
            else
                unlisten = stop;
        });
        return () => {
            cancelled = true;
            unlisten?.();
        };
    }, []);
    // Keep the <html> class and the saved preference in sync with the chosen theme.
    useEffect(() => {
        document.documentElement.classList.toggle('dark', dark);
        localStorage.setItem('theme', dark ? 'dark' : 'light');
    }, [dark]);
    // Auto-hide the level-up / XP notice.
    useEffect(() => {
        if (!notice)
            return;
        const id = window.setTimeout(() => setNotice(null), 3500);
        return () => window.clearTimeout(id);
    }, [notice]);
    function applyXp(outcome) {
        setProfile(outcome.profile);
        if (outcome.leveledUp) {
            setNotice(`Level up! You reached level ${outcome.profile.level}.`);
        }
        else if (outcome.xpDelta > 0) {
            setNotice(`+${outcome.xpDelta} XP`);
        }
    }
    const createTask = (input) => run(async () => {
        const task = await api.createTask(input);
        setTasks((prev) => [task, ...prev]);
    });
    const toggleTask = (task) => run(async () => {
        const result = await api.setTaskStatus(task.id, task.status === 'done' ? 'todo' : 'done');
        setTasks((prev) => prev.map((t) => (t.id === result.task.id ? result.task : t)));
        applyXp(result.xp);
        await refreshQuests();
    });
    const deleteTask = (task) => run(async () => {
        await api.deleteTask(task.id);
        setTasks((prev) => prev.filter((t) => t.id !== task.id));
    });
    const createHabit = (input) => run(async () => {
        await api.createHabit(input);
        setHabits(await api.listHabits(localToday()));
    });
    const toggleHabit = (habit) => run(async () => {
        const result = await api.toggleHabit(habit.id, localToday());
        setHabits((prev) => prev.map((h) => h.id === habit.id
            ? {
                ...h,
                completedToday: result.completed,
                currentStreak: result.currentStreak,
            }
            : h));
        applyXp(result.xp);
        await refreshQuests();
    });
    const deleteHabit = (habit) => run(async () => {
        await api.deleteHabit(habit.id);
        setHabits((prev) => prev.filter((h) => h.id !== habit.id));
    });
    const createGoal = (input) => run(async () => {
        const goal = await api.createGoal(input);
        setGoals((prev) => [goal, ...prev]);
    });
    const setGoalStatus = (goal, status) => run(async () => {
        const result = await api.setGoalStatus(goal.id, status);
        setGoals((prev) => prev.map((g) => (g.id === result.goal.id ? result.goal : g)));
        applyXp(result.xp);
    });
    const deleteGoal = (goal) => run(async () => {
        await api.deleteGoal(goal.id);
        setGoals((prev) => prev.filter((g) => g.id !== goal.id));
    });
    const addGoalMilestone = (goal, input) => run(async () => {
        const updated = await api.addGoalMilestone(goal.id, input);
        setGoals((prev) => prev.map((g) => (g.id === updated.id ? updated : g)));
    });
    const toggleGoalMilestone = (milestone) => run(async () => {
        const status = milestone.status === 'completed' ? 'active' : 'completed';
        const updated = await api.setGoalMilestoneStatus(milestone.id, status);
        setGoals((prev) => prev.map((g) => (g.id === updated.id ? updated : g)));
    });
    const deleteGoalMilestone = (milestone) => run(async () => {
        const updated = await api.deleteGoalMilestone(milestone.id);
        setGoals((prev) => prev.map((g) => (g.id === updated.id ? updated : g)));
    });
    const createBlock = (input) => run(async () => {
        const block = await api.createBlock(input);
        setBlocks((prev) => [...prev, block]);
    });
    const setBlockStatus = (block, status) => run(async () => {
        const updated = await api.setBlockStatus(block.id, status);
        setBlocks((prev) => prev.map((b) => (b.id === updated.id ? updated : b)));
    });
    const deleteBlock = (block) => run(async () => {
        await api.deleteBlock(block.id);
        setBlocks((prev) => prev.filter((b) => b.id !== block.id));
    });
    // Starting returns the new session so the timer can hold its id; errors surface in the banner.
    const startFocus = async (input) => {
        try {
            setError(null);
            const session = await api.startFocusSession(input);
            setFocusSessions((prev) => [session, ...prev]);
            return session;
        }
        catch (e) {
            setError(e instanceof Error ? e.message : String(e));
            return null;
        }
    };
    const refreshFocusStats = async () => setFocusStats(await api.focusStats());
    const completeFocus = (id, actualDurationSeconds) => run(async () => {
        const result = await api.completeFocusSession(id, actualDurationSeconds);
        setFocusSessions((prev) => prev.map((s) => (s.id === result.session.id ? result.session : s)));
        applyXp(result.xp);
        await refreshFocusStats();
    });
    const cancelFocus = (id) => run(async () => {
        const result = await api.cancelFocusSession(id);
        setFocusSessions((prev) => prev.map((s) => (s.id === result.session.id ? result.session : s)));
        await refreshFocusStats();
    });
    const deleteFocus = (session) => run(async () => {
        await api.deleteFocusSession(session.id);
        setFocusSessions((prev) => prev.filter((s) => s.id !== session.id));
        await refreshFocusStats();
        // Deleting a completed session refunds XP, so refresh the profile too.
        setProfile(await api.getProfile());
    });
    // Send a chat turn. Actions the assistant takes (create/complete) write to the
    // database, so anything it may have touched is reloaded afterwards.
    const sendAssistant = async (text) => {
        setAssistantBusy(true);
        setAssistantStream('');
        setPendingUser(text);
        try {
            setError(null);
            const result = await api.chatStream({ conversationId: activeConversation, message: text });
            setAssistantMessages((prev) => [...prev, ...result.messages]);
            setActiveConversation(result.conversation.id);
            setConversations((prev) => [
                result.conversation,
                ...prev.filter((c) => c.id !== result.conversation.id),
            ]);
            applyXp(result.xp);
            await load();
        }
        catch (e) {
            setError(e instanceof Error ? e.message : String(e));
        }
        finally {
            setAssistantBusy(false);
            setAssistantStream('');
            setPendingUser(null);
        }
    };
    const selectConversation = (id) => run(async () => {
        setAssistantMessages(await api.getConversation(id));
        setActiveConversation(id);
    });
    function newConversation() {
        setActiveConversation(null);
        setAssistantMessages([]);
    }
    const deleteConversation = (id) => run(async () => {
        await api.deleteConversation(id);
        setConversations((prev) => prev.filter((c) => c.id !== id));
        if (activeConversation === id) {
            setActiveConversation(null);
            setAssistantMessages([]);
        }
    });
    const saveAiSettings = (input) => run(async () => {
        setAiSettings(await api.setAiSettings(input));
    });
    function toggleTheme() {
        setDark((value) => !value);
    }
    return (_jsxs("div", { className: "app", "data-collapsed": collapsed, children: [_jsxs("aside", { className: "sidebar", style: collapsed ? { display: 'none' } : undefined, children: [_jsx("div", { className: "brand", children: "Habit Planner" }), _jsx("nav", { "aria-label": "Main", children: NAV_ITEMS.map((item) => (_jsxs("button", { type: "button", "aria-current": view === item.id ? 'page' : undefined, onClick: () => setView(item.id), children: [item.icon, _jsx("span", { children: item.label }), item.soon && _jsx("span", { "data-soon": true, children: "soon" })] }, item.id))) }), _jsx("div", { className: "spacer" }), _jsxs("button", { type: "button", onClick: toggleTheme, "aria-label": "Switch theme", children: [dark ? _jsx(Sun, { "aria-hidden": true }) : _jsx(Moon, { "aria-hidden": true }), _jsx("span", { children: dark ? 'Light theme' : 'Dark theme' })] }), _jsx("button", { type: "button", onClick: () => setCollapsed(!collapsed), "aria-expanded": !collapsed, "aria-label": collapsed ? 'Expand sidebar' : 'Collapse sidebar', children: collapsed ? 'Expand' : 'Collapse' })] }), _jsxs("main", { className: "main", children: [view === 'today' && (_jsx(TodayPage, { tasks: tasks, habits: habits, profile: profile ?? FALLBACK_PROFILE })), view === 'tasks' && (_jsx(TasksPage, { tasks: tasks, onCreate: createTask, onToggle: toggleTask, onDelete: deleteTask })), view === 'habits' && (_jsx(HabitsPage, { habits: habits, onCreate: createHabit, onToggle: toggleHabit, onDelete: deleteHabit })), view === 'quests' && _jsx(QuestsPage, { quests: quests, achievements: achievements }), view === 'goals' && (_jsx(GoalsPage, { goals: goals, onCreate: createGoal, onSetStatus: setGoalStatus, onDelete: deleteGoal, onAddMilestone: addGoalMilestone, onToggleMilestone: toggleGoalMilestone, onDeleteMilestone: deleteGoalMilestone })), view === 'calendar' && (_jsx(CalendarPage, { blocks: blocks, tasks: tasks, onCreate: createBlock, onSetStatus: setBlockStatus, onDelete: deleteBlock })), view === 'focus' && (_jsx(FocusPage, { sessions: focusSessions, stats: focusStats, tasks: tasks, onStart: startFocus, onComplete: completeFocus, onCancel: cancelFocus, onDelete: deleteFocus })), COMING_SOON[view] && (_jsx(PlaceholderPage, { title: COMING_SOON[view].title, body: COMING_SOON[view].body }))] }), _jsx(Suspense, { fallback: _jsx("aside", { className: "assistant", "aria-hidden": true }), children: _jsx(JarvisPanel, { messages: assistantMessages, conversations: conversations, activeConversationId: activeConversation, busy: assistantBusy, streaming: assistantStream, pendingUser: pendingUser, settings: aiSettings, onSend: sendAssistant, onSelectConversation: selectConversation, onNewConversation: newConversation, onDeleteConversation: deleteConversation, onSaveSettings: saveAiSettings }) }), error && (_jsxs("div", { role: "alert", style: {
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
                }, children: [_jsx("span", { children: error }), _jsx("button", { type: "button", onClick: () => setError(null), "aria-label": "Dismiss error", children: _jsx(X, { "aria-hidden": true }) })] })), notice && (_jsx("div", { className: "toast show", role: "status", children: notice }))] }));
}
const FALLBACK_PROFILE = {
    level: 1,
    totalXp: 0,
    xpIntoLevel: 0,
    xpForNext: 100,
};
