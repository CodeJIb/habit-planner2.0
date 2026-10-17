import { jsx as _jsx, jsxs as _jsxs } from "react/jsx-runtime";
import { Sparkles } from 'lucide-react';
import { useState } from 'react';
import { Progress } from './ui/progress';
import { localToday } from '../lib/dates';
function greetingFor(hour) {
    if (hour < 12)
        return 'Good morning';
    if (hour < 18)
        return 'Good afternoon';
    return 'Good evening';
}
/** Open tasks: everything not done yet, with overdue tasks first. */
function todaysOpen(tasks) {
    const open = tasks.filter((t) => t.status !== 'done' && t.status !== 'archived');
    return open.sort((a, b) => {
        const da = a.dueAt ? a.dueAt.slice(0, 10) : '9999';
        const db = b.dueAt ? b.dueAt.slice(0, 10) : '9999';
        return da.localeCompare(db);
    });
}
export function TodayPage({ tasks, habits, profile }) {
    // Greeting and date are captured once so they stay stable across re-renders.
    const [header] = useState(() => {
        const now = new Date();
        return {
            greeting: greetingFor(now.getHours()),
            date: now.toLocaleDateString(undefined, { weekday: 'long', day: 'numeric', month: 'long' }),
        };
    });
    const openTasks = todaysOpen(tasks);
    const openHabits = habits.filter((h) => !h.completedToday);
    const best = bestStreak(habits);
    const pct = Math.min(100, Math.round((profile.xpIntoLevel / profile.xpForNext) * 100));
    const circumference = 2 * Math.PI * 40;
    return (_jsxs("div", { className: "screen", children: [_jsx("h1", { children: header.greeting }), _jsxs("p", { className: "sub", children: [header.date, " \u00B7", ' ', openTasks.length, " tasks and ", openHabits.length, " habits left to check in."] }), _jsxs("div", { className: "card", "aria-label": "Level progress", children: [_jsxs("div", { style: { display: 'flex', alignItems: 'center', gap: '1.25rem', flexWrap: 'wrap' }, children: [_jsxs("div", { style: {
                                    position: 'relative',
                                    width: '7rem',
                                    height: '7rem',
                                    flex: 'none',
                                    borderRadius: '9999px',
                                }, children: [_jsxs("svg", { viewBox: "0 0 100 100", width: "100", height: "100", "aria-hidden": true, children: [_jsx("circle", { cx: "50", cy: "50", r: "40", fill: "none", stroke: "var(--color-accent)", strokeWidth: "10", opacity: "0.25" }), _jsx("circle", { cx: "50", cy: "50", r: "40", fill: "none", stroke: "var(--color-xp)", strokeWidth: "10", strokeLinecap: "round", strokeDasharray: circumference, strokeDashoffset: circumference * (1 - pct / 100), transform: "rotate(-90 50 50)" })] }), _jsx("div", { style: {
                                            position: 'absolute',
                                            inset: 0,
                                            display: 'grid',
                                            placeItems: 'center',
                                            textAlign: 'center',
                                        }, children: _jsxs("div", { children: [_jsxs("p", { style: { margin: 0, fontSize: '1.4rem', fontWeight: 700, lineHeight: 1 }, children: ["Level ", profile.level] }), _jsx("span", { style: { fontSize: '0.7rem', color: 'var(--color-muted)' }, children: "current" })] }) })] }), _jsxs("div", { className: "grow", style: { minWidth: 0 }, children: [_jsx("h2", { style: { margin: '0 0 0.25rem', fontSize: '1.125rem' }, children: "Level progress" }), _jsx(Progress, { value: pct, "aria-label": "XP progress to next level" }), _jsxs("p", { className: "small", style: { marginTop: '0.375rem' }, children: [profile.xpIntoLevel, " of ", profile.xpForNext, " XP to level ", profile.level + 1] })] })] }), _jsxs("div", { style: {
                            display: 'flex',
                            flexWrap: 'wrap',
                            gap: '1.5rem',
                            marginTop: '1rem',
                            color: 'var(--color-muted)',
                            fontSize: '0.8125rem',
                        }, children: [_jsxs("span", { children: ["Total XP: ", _jsx("b", { style: { color: 'var(--color-foreground)' }, children: profile.totalXp })] }), _jsxs("span", { children: ["Best streak today: ", _jsx("b", { style: { color: 'var(--color-foreground)' }, children: best }), " day", best === 1 ? '' : 's'] }), _jsxs("span", { children: ["Open tasks: ", _jsx("b", { style: { color: 'var(--color-foreground)' }, children: openTasks.length })] })] })] }), _jsxs("div", { className: "cols", style: { display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '1.25rem' }, children: [_jsxs("section", { className: "card", "aria-label": "Today's tasks", children: [_jsx("h2", { children: "Today's tasks" }), openTasks.length === 0 ? (_jsx("p", { style: { color: 'var(--color-muted)' }, children: "Nothing left to do. Nice work." })) : (_jsx("ul", { style: { listStyle: 'none', margin: 0, padding: 0 }, children: openTasks.slice(0, 8).map((task) => (_jsx(TodayTaskRow, { task: task }, task.id))) }))] }), _jsxs("section", { className: "card", "aria-label": "Today's habits", children: [_jsx("h2", { children: "Habits" }), habits.length === 0 ? (_jsx("p", { style: { color: 'var(--color-muted)' }, children: "No habits yet. Add one from the Habits screen." })) : (_jsx("ul", { style: { listStyle: 'none', margin: 0, padding: 0 }, children: habits.map((habit) => (_jsx(TodayHabitRow, { habit: habit }, habit.id))) }))] })] })] }));
}
function bestStreak(habits) {
    return habits.reduce((max, h) => Math.max(max, h.currentStreak), 0);
}
function TodayTaskRow({ task }) {
    const overdue = task.dueAt !== null && task.dueAt.slice(0, 10) < localToday();
    return (_jsxs("li", { className: "row", "data-task-row": task.id, children: [_jsxs("div", { className: "grow", children: [_jsx("b", { children: task.title }), task.dueAt !== null && (_jsxs("span", { className: "small", children: [overdue ? 'Overdue · ' : 'Due ', task.dueAt.slice(0, 10)] }))] }), _jsx("span", { className: `tag ${task.priority}`, children: task.priority })] }));
}
function TodayHabitRow({ habit }) {
    return (_jsxs("li", { className: "row", children: [_jsx(Sparkles, { "aria-hidden": true, className: "text-xp" }), _jsxs("div", { className: "grow", children: [_jsx("b", { children: habit.title }), _jsx("span", { className: "small", children: habit.completedToday
                            ? 'Checked in today'
                            : `Checked in ${habit.currentStreak} day streak${habit.currentStreak === 1 ? '' : 's'}` })] }), habit.completedToday ? (_jsx("span", { className: "tag", style: { background: 'var(--color-ok)', color: '#fff' }, children: "Done" })) : (_jsx("span", { className: "small", children: "Not yet" }))] }));
}
