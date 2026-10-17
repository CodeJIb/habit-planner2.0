import { jsx as _jsx, jsxs as _jsxs } from "react/jsx-runtime";
import { Check, Circle, Flame, Plus, Trash2 } from 'lucide-react';
import { useState } from 'react';
import { Button } from './ui/button';
import { Card } from './ui/card';
import { Input } from './ui/input';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue, } from './ui/select';
import { ConfirmDialog } from './ConfirmDialog';
import { formatLocalDate, localToday } from '../lib/dates';
/** The seven day buttons of the current week, in calendar order. */
function weekDays() {
    const today = localToday();
    const start = new Date(`${today}T00:00:00`);
    // Monday-first week containing today.
    const offset = (start.getDay() + 6) % 7;
    start.setDate(start.getDate() - offset);
    const labels = ['M', 'T', 'W', 'T', 'F', 'S', 'S'];
    return Array.from({ length: 7 }, (_, i) => {
        const day = new Date(start);
        day.setDate(start.getDate() + i);
        return { label: labels[i], date: formatLocalDate(day) };
    });
}
export function HabitsPage({ habits, onCreate, onToggle, onDelete }) {
    const [title, setTitle] = useState('');
    const [frequency, setFrequency] = useState('daily');
    const [busy, setBusy] = useState(false);
    const [pendingDelete, setPendingDelete] = useState(null);
    async function submit(e) {
        e.preventDefault();
        const trimmed = title.trim();
        if (!trimmed || busy)
            return;
        setBusy(true);
        try {
            await onCreate({ title: trimmed, frequencyType: frequency });
            setTitle('');
        }
        finally {
            setBusy(false);
        }
    }
    const week = weekDays();
    const todayIso = localToday();
    return (_jsxs("div", { children: [_jsx("h1", { children: "Habits" }), _jsx("p", { className: "sub", children: "Check in once a day to keep your streak. The whole week is shown." }), _jsx(Card, { children: _jsxs("form", { onSubmit: submit, className: "flex flex-wrap items-center gap-2", children: [_jsx(Input, { value: title, onChange: (e) => setTitle(e.target.value), placeholder: "New habit\u2026", maxLength: 200, "aria-label": "Habit title", className: "min-w-48 flex-1" }), _jsxs(Select, { value: frequency, onValueChange: (v) => setFrequency(v), children: [_jsx(SelectTrigger, { "aria-label": "Frequency", className: "w-40", children: _jsx(SelectValue, {}) }), _jsxs(SelectContent, { children: [_jsx(SelectItem, { value: "daily", children: "Daily" }), _jsx(SelectItem, { value: "weekly", children: "Weekly" }), _jsx(SelectItem, { value: "custom", children: "Custom" })] })] }), _jsxs(Button, { type: "submit", disabled: !title.trim() || busy, children: [_jsx(Plus, { className: "h-4 w-4", "aria-hidden": true }), " Add"] })] }) }), _jsx("div", { style: { display: 'flex', flexDirection: 'column', gap: '0.25rem', marginTop: '1rem' }, children: habits.length === 0 ? (_jsx("p", { style: { padding: '2rem 0', textAlign: 'center', color: 'var(--color-muted)' }, children: "No habits yet. Add your first one above." })) : (habits.map((habit) => (_jsxs(Card, { className: "row", children: [_jsx("button", { type: "button", onClick: () => onToggle(habit), "aria-pressed": habit.completedToday, "aria-label": habit.completedToday ? `Undo check-in: ${habit.title}` : `Check in today: ${habit.title}`, className: "text-muted hover:text-success", children: habit.completedToday ? (_jsx(Check, { className: "h-5 w-5 text-success" })) : (_jsx(Circle, { className: "h-5 w-5" })) }), _jsxs("div", { className: "grow", style: { minWidth: 0 }, children: [_jsx("b", { children: habit.title }), _jsxs("span", { className: "small", children: ["Current streak: ", habit.currentStreak, " day", habit.currentStreak === 1 ? '' : 's'] }), _jsx("div", { className: "week", "aria-label": `This week for ${habit.title}`, children: week.map((day) => {
                                        const done = day.date === todayIso && habit.completedToday;
                                        const isToday = day.date === todayIso;
                                        return (_jsx("button", { type: "button", disabled: !isToday, "aria-pressed": done, "aria-label": isToday
                                                ? done
                                                    ? `Checked in today: ${habit.title}`
                                                    : `Check in ${habit.title} today`
                                                : undefined, title: isToday ? (done ? 'Checked in today' : 'Check in today') : 'Only today can be checked', children: day.label }, day.date));
                                    }) })] }), _jsxs("span", { className: "xp-pill", children: [_jsx(Flame, { "aria-hidden": true, className: "mr-1 inline h-3 w-3" }), habit.currentStreak] }), _jsx(Button, { variant: "outline", size: "sm", onClick: () => setPendingDelete(habit), "aria-label": `Delete "${habit.title}"`, children: _jsx(Trash2, { "aria-hidden": true }) })] }, habit.id)))) }), pendingDelete && (_jsx(ConfirmDialog, { title: `Delete "${pendingDelete.title}"?`, body: "This habit will be permanently deleted. This cannot be undone.", onCancel: () => setPendingDelete(null), onConfirm: () => {
                    onDelete(pendingDelete);
                    setPendingDelete(null);
                } }))] }));
}
