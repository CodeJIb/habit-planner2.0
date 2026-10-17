import { jsx as _jsx, jsxs as _jsxs, Fragment as _Fragment } from "react/jsx-runtime";
import { Pause, Play, Square, Trash2 } from 'lucide-react';
import { useEffect, useState } from 'react';
import { Badge } from './ui/badge';
import { Button } from './ui/button';
import { Card } from './ui/card';
import { Progress } from './ui/progress';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue, } from './ui/select';
import { ConfirmDialog } from './ConfirmDialog';
const DURATIONS = [15, 25, 45, 60];
function formatClock(seconds) {
    const mm = String(Math.floor(seconds / 60)).padStart(2, '0');
    const ss = String(seconds % 60).padStart(2, '0');
    return `${mm}:${ss}`;
}
/** Seconds a completed session focused, for the history line. */
function focusedMinutes(session) {
    return Math.round((session.actualDurationSeconds ?? 0) / 60);
}
/** A Pomodoro timer with a running session log and daily/weekly totals. */
export function FocusPage({ sessions, stats, tasks, onStart, onComplete, onCancel, onDelete }) {
    const [duration, setDuration] = useState(25);
    const [taskId, setTaskId] = useState(null);
    const [session, setSession] = useState(null);
    const [secondsLeft, setSecondsLeft] = useState(duration * 60);
    const [running, setRunning] = useState(false);
    const [busy, setBusy] = useState(false);
    const [pendingDelete, setPendingDelete] = useState(null);
    // Tick the countdown while a session is running.
    useEffect(() => {
        if (!running || !session)
            return;
        const id = window.setInterval(() => setSecondsLeft((s) => Math.max(0, s - 1)), 1000);
        return () => window.clearInterval(id);
    }, [running, session]);
    // Auto-finish the moment the countdown hits zero.
    useEffect(() => {
        if (!running || secondsLeft > 0 || !session)
            return;
        // oxlint-disable-next-line react/set-state-in-effect
        setRunning(false);
        const focused = Math.max(0, session.plannedDurationMinutes * 60 - secondsLeft);
        void onComplete(session.id, focused).finally(() => {
            setSession(null);
            setSecondsLeft(duration * 60);
        });
    }, [running, secondsLeft, session, duration, onComplete]);
    async function start() {
        if (busy || session)
            return;
        setBusy(true);
        try {
            const created = await onStart({ taskId, plannedDurationMinutes: duration });
            if (created) {
                setSession(created);
                setSecondsLeft(created.plannedDurationMinutes * 60);
                setRunning(true);
            }
        }
        finally {
            setBusy(false);
        }
    }
    async function stop() {
        if (busy || !session)
            return;
        setBusy(true);
        try {
            await onCancel(session.id);
        }
        finally {
            setRunning(false);
            setSession(null);
            setSecondsLeft(duration * 60);
            setBusy(false);
        }
    }
    function taskTitle(id) {
        if (id === null)
            return 'No task';
        return tasks.find((t) => t.id === id)?.title ?? 'Deleted task';
    }
    const plannedSeconds = session ? session.plannedDurationMinutes * 60 : duration * 60;
    const pct = plannedSeconds > 0 ? Math.round(((plannedSeconds - secondsLeft) / plannedSeconds) * 100) : 0;
    return (_jsxs("div", { className: "screen", children: [_jsx("h1", { children: "Focus" }), _jsx("p", { className: "sub", children: "Run a Pomodoro, stay on task, and log every focused minute." }), _jsxs("div", { style: { display: 'flex', flexWrap: 'wrap', gap: '1.5rem', marginBottom: '1rem' }, children: [_jsxs("span", { className: "xp-pill", "aria-label": "Focus time today", children: ["Today \u00B7 ", stats?.todayMinutes ?? 0, " min (", stats?.todaySessions ?? 0, ")"] }), _jsxs("span", { className: "xp-pill", "aria-label": "Focus time this week", children: ["This week \u00B7 ", stats?.weekMinutes ?? 0, " min (", stats?.weekSessions ?? 0, ")"] })] }), _jsxs(Card, { children: [_jsx("div", { style: {
                            textAlign: 'center',
                            fontFamily: 'var(--font-display)',
                            fontSize: '3.5rem',
                            fontWeight: 700,
                            fontVariantNumeric: 'tabular-nums',
                            lineHeight: 1.1,
                        }, "aria-live": "polite", children: formatClock(secondsLeft) }), _jsx("div", { style: { margin: '0.75rem 0 1.25rem' }, children: _jsx(Progress, { value: pct, "aria-label": "Session progress" }) }), _jsxs("div", { className: "flex flex-wrap items-center justify-center gap-2", children: [_jsxs("label", { className: "flex items-center gap-2", children: [_jsx("span", { className: "small", children: "Length" }), _jsxs(Select, { value: String(duration), onValueChange: (v) => {
                                            const minutes = Number(v);
                                            setDuration(minutes);
                                            if (!session)
                                                setSecondsLeft(minutes * 60);
                                        }, disabled: !!session, children: [_jsx(SelectTrigger, { "aria-label": "Session length", className: "w-28", size: "sm", children: _jsx(SelectValue, {}) }), _jsx(SelectContent, { children: DURATIONS.map((minutes) => (_jsxs(SelectItem, { value: String(minutes), children: [minutes, " min"] }, minutes))) })] })] }), _jsxs("label", { className: "flex items-center gap-2", children: [_jsx("span", { className: "small", children: "Task" }), _jsxs(Select, { value: taskId === null ? 'none' : String(taskId), onValueChange: (v) => setTaskId(v === 'none' ? null : Number(v)), disabled: !!session, children: [_jsx(SelectTrigger, { "aria-label": "Link a task", className: "w-44", size: "sm", children: _jsx(SelectValue, {}) }), _jsxs(SelectContent, { children: [_jsx(SelectItem, { value: "none", children: "No task" }), tasks.map((task) => (_jsx(SelectItem, { value: String(task.id), children: task.title }, task.id)))] })] })] })] }), _jsx("div", { className: "flex items-center justify-center gap-2", style: { marginTop: '1.25rem' }, children: !session ? (_jsxs(Button, { onClick: start, disabled: busy, children: [_jsx(Play, { className: "h-4 w-4", "aria-hidden": true }), " Start"] })) : (_jsxs(_Fragment, { children: [_jsx(Button, { onClick: () => setRunning((r) => !r), disabled: busy, children: running ? (_jsxs(_Fragment, { children: [_jsx(Pause, { className: "h-4 w-4", "aria-hidden": true }), " Pause"] })) : (_jsxs(_Fragment, { children: [_jsx(Play, { className: "h-4 w-4", "aria-hidden": true }), " Resume"] })) }), _jsxs(Button, { variant: "outline", onClick: stop, disabled: busy, children: [_jsx(Square, { className: "h-4 w-4", "aria-hidden": true }), " Give up"] })] })) })] }), _jsx("h2", { style: { marginTop: '1.5rem' }, children: "History" }), sessions.length === 0 ? (_jsx("p", { style: { padding: '2rem 0', textAlign: 'center', color: 'var(--color-muted)' }, children: "No focus sessions yet. Start the timer above." })) : (_jsx("div", { style: { display: 'flex', flexDirection: 'column', gap: '0.25rem' }, children: sessions.map((item) => {
                    const completed = item.status === 'completed';
                    return (_jsxs(Card, { className: "row", children: [_jsx("span", { className: "xp-pill", style: { minWidth: '3.5rem' }, children: item.startedAt.slice(11, 16) }), _jsxs("div", { className: "grow", style: { minWidth: 0 }, children: [_jsx("b", { children: taskTitle(item.taskId) }), _jsx("span", { className: "small", children: completed
                                            ? `Focused ${focusedMinutes(item)} of ${item.plannedDurationMinutes} min`
                                            : `Planned ${item.plannedDurationMinutes} min` })] }), _jsx(Badge, { variant: completed ? 'secondary' : 'outline', children: completed ? 'Completed' : 'Cancelled' }), _jsx(Button, { variant: "outline", size: "sm", onClick: () => setPendingDelete(item), "aria-label": `Delete session at ${item.startedAt.slice(11, 16)}`, children: _jsx(Trash2, { "aria-hidden": true }) })] }, item.id));
                }) })), pendingDelete && (_jsx(ConfirmDialog, { title: "Delete this session?", body: "This focus session will be permanently deleted and any XP it earned is removed. This cannot be undone.", onCancel: () => setPendingDelete(null), onConfirm: () => {
                    onDelete(pendingDelete);
                    setPendingDelete(null);
                } }))] }));
}
