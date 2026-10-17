import { jsx as _jsx, jsxs as _jsxs } from "react/jsx-runtime";
import { ChevronLeft, ChevronRight, Plus, Trash2 } from 'lucide-react';
import { useState } from 'react';
import { Badge } from './ui/badge';
import { Button } from './ui/button';
import { Card } from './ui/card';
import { Input } from './ui/input';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue, } from './ui/select';
import { ConfirmDialog } from './ConfirmDialog';
import { addDays, localToday, mondayOf } from '../lib/dates';
const BLOCK_LABEL = {
    task: 'Task',
    break: 'Break',
    other: 'Other',
};
const BLOCK_VARIANT = {
    task: 'default',
    break: 'secondary',
    other: 'outline',
};
/** "2026-10-16" -> "Fri, Oct 16" for the day header. */
function dayLabel(date) {
    const [y, m, d] = date.split('-').map(Number);
    return new Date(y, m - 1, d).toLocaleDateString(undefined, {
        weekday: 'short',
        month: 'short',
        day: 'numeric',
    });
}
/** Blocks that overlap the given day (a block spanning midnight counts too). */
function blocksForDay(blocks, day) {
    return blocks
        .filter((b) => b.startAt.slice(0, 10) <= day && b.endAt.slice(0, 10) >= day)
        .sort((a, b) => a.startAt.localeCompare(b.startAt));
}
/** "2026-10-16T09:00" -> "09:00". */
function timeOf(iso) {
    return iso.slice(11, 16);
}
/** The Calendar screen: pick a day, see its blocks, schedule new ones. */
export function CalendarPage({ blocks, tasks, onCreate, onSetStatus, onDelete }) {
    const today = localToday();
    const [weekStart, setWeekStart] = useState(() => mondayOf(today));
    const [selectedDay, setSelectedDay] = useState(today);
    const [title, setTitle] = useState('');
    const [startTime, setStartTime] = useState('09:00');
    const [endTime, setEndTime] = useState('10:00');
    const [blockType, setBlockType] = useState('task');
    const [taskId, setTaskId] = useState(null);
    const [busy, setBusy] = useState(false);
    const [pendingDelete, setPendingDelete] = useState(null);
    const week = Array.from({ length: 7 }, (_, i) => addDays(weekStart, i));
    const dayBlocks = blocksForDay(blocks, selectedDay);
    function shiftWeek(days) {
        setWeekStart((prev) => addDays(prev, days));
        setSelectedDay((prev) => addDays(prev, days));
    }
    function goToday() {
        setWeekStart(mondayOf(today));
        setSelectedDay(today);
    }
    async function submit(e) {
        e.preventDefault();
        const trimmed = title.trim();
        if (!trimmed || busy)
            return;
        setBusy(true);
        try {
            await onCreate({
                title: trimmed,
                startAt: `${selectedDay}T${startTime}`,
                endAt: `${selectedDay}T${endTime}`,
                taskId,
                blockType,
            });
            setTitle('');
        }
        finally {
            setBusy(false);
        }
    }
    return (_jsxs("div", { className: "screen", children: [_jsx("h1", { children: "Calendar" }), _jsx("p", { className: "sub", children: "Plan your day in time blocks. A block can be tied to a task." }), _jsx(Card, { children: _jsxs("form", { onSubmit: submit, className: "flex flex-wrap items-end gap-2", children: [_jsxs("label", { className: "flex flex-col gap-1", children: [_jsx("span", { className: "small", children: "Title" }), _jsx(Input, { value: title, onChange: (e) => setTitle(e.target.value), placeholder: "Block title\u2026", maxLength: 200, "aria-label": "Block title", className: "min-w-40 flex-1" })] }), _jsxs("label", { className: "flex flex-col gap-1", children: [_jsx("span", { className: "small", children: "Start" }), _jsx(Input, { type: "time", value: startTime, onChange: (e) => setStartTime(e.target.value), "aria-label": "Start time", className: "w-32" })] }), _jsxs("label", { className: "flex flex-col gap-1", children: [_jsx("span", { className: "small", children: "End" }), _jsx(Input, { type: "time", value: endTime, onChange: (e) => setEndTime(e.target.value), "aria-label": "End time", className: "w-32" })] }), _jsxs("label", { className: "flex flex-col gap-1", children: [_jsx("span", { className: "small", children: "Type" }), _jsxs(Select, { value: blockType, onValueChange: (v) => setBlockType(v), children: [_jsx(SelectTrigger, { "aria-label": "Block type", className: "w-32", children: _jsx(SelectValue, {}) }), _jsxs(SelectContent, { children: [_jsx(SelectItem, { value: "task", children: "Task" }), _jsx(SelectItem, { value: "break", children: "Break" }), _jsx(SelectItem, { value: "other", children: "Other" })] })] })] }), _jsxs("label", { className: "flex flex-col gap-1", children: [_jsx("span", { className: "small", children: "Task" }), _jsxs(Select, { value: taskId === null ? 'none' : String(taskId), onValueChange: (v) => {
                                        if (v === 'none') {
                                            setTaskId(null);
                                        }
                                        else {
                                            const id = Number(v);
                                            setTaskId(id);
                                            const task = tasks.find((t) => t.id === id);
                                            if (task)
                                                setTitle(task.title);
                                        }
                                    }, children: [_jsx(SelectTrigger, { "aria-label": "Link a task", className: "w-40", children: _jsx(SelectValue, {}) }), _jsxs(SelectContent, { children: [_jsx(SelectItem, { value: "none", children: "None" }), tasks.map((task) => (_jsx(SelectItem, { value: String(task.id), children: task.title }, task.id)))] })] })] }), _jsxs(Button, { type: "submit", disabled: !title.trim() || busy, children: [_jsx(Plus, { className: "h-4 w-4", "aria-hidden": true }), " Add"] })] }) }), _jsxs("div", { style: {
                    display: 'flex',
                    alignItems: 'center',
                    gap: '0.5rem',
                    marginTop: '1.25rem',
                    marginBottom: '0.75rem',
                }, children: [_jsx(Button, { variant: "outline", size: "sm", onClick: () => shiftWeek(-1), "aria-label": "Previous week", children: _jsx(ChevronLeft, { "aria-hidden": true }) }), _jsx("b", { style: { minWidth: '9rem', textAlign: 'center' }, children: dayLabel(weekStart) }), _jsx(Button, { variant: "outline", size: "sm", onClick: () => shiftWeek(1), "aria-label": "Next week", children: _jsx(ChevronRight, { "aria-hidden": true }) }), _jsx("div", { className: "grow" }), _jsx(Button, { variant: "ghost", size: "sm", onClick: goToday, children: "Today" })] }), _jsx("div", { className: "week", "aria-label": "Days of the week", children: week.map((date) => {
                    const isToday = date === today;
                    const [y, m, d] = date.split('-').map(Number);
                    return (_jsxs("button", { type: "button", "aria-pressed": date === selectedDay, onClick: () => setSelectedDay(date), "aria-label": `Show ${dayLabel(date)}`, title: dayLabel(date), style: { width: '2.5rem', height: '2.5rem', fontSize: '0.75rem' }, children: [new Date(y, m - 1, d).getDate(), isToday ? '•' : ''] }, date));
                }) }), _jsx("h2", { style: { marginTop: '1.25rem' }, children: dayLabel(selectedDay) }), dayBlocks.length === 0 ? (_jsx("p", { style: { padding: '2rem 0', textAlign: 'center', color: 'var(--color-muted)' }, children: "No blocks scheduled for this day." })) : (_jsx("div", { style: { display: 'flex', flexDirection: 'column', gap: '0.25rem' }, children: dayBlocks.map((block) => (_jsxs(Card, { className: "row", children: [_jsxs("span", { className: "xp-pill", style: { minWidth: '4.5rem' }, children: [timeOf(block.startAt), "\u2013", timeOf(block.endAt)] }), _jsxs("div", { className: "grow", style: { minWidth: 0 }, children: [_jsx("b", { children: block.title }), block.taskId !== null && _jsx("span", { className: "small", children: "Linked to a task" })] }), _jsx(Badge, { variant: BLOCK_VARIANT[block.blockType], children: BLOCK_LABEL[block.blockType] }), _jsxs(Select, { value: block.status, onValueChange: (v) => void onSetStatus(block, v), children: [_jsx(SelectTrigger, { "aria-label": `Status for ${block.title}`, className: "w-32", size: "sm", children: _jsx(SelectValue, {}) }), _jsxs(SelectContent, { children: [_jsx(SelectItem, { value: "scheduled", children: "Scheduled" }), _jsx(SelectItem, { value: "running", children: "Running" }), _jsx(SelectItem, { value: "done", children: "Done" }), _jsx(SelectItem, { value: "cancelled", children: "Cancelled" })] })] }), _jsx(Button, { variant: "outline", size: "sm", onClick: () => setPendingDelete(block), "aria-label": `Delete block "${block.title}"`, children: _jsx(Trash2, { "aria-hidden": true }) })] }, block.id))) })), pendingDelete && (_jsx(ConfirmDialog, { title: `Delete "${pendingDelete.title}"?`, body: "This block will be permanently deleted. This cannot be undone.", onCancel: () => setPendingDelete(null), onConfirm: () => {
                    onDelete(pendingDelete);
                    setPendingDelete(null);
                } }))] }));
}
