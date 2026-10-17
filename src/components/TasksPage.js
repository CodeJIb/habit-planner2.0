import { jsx as _jsx, jsxs as _jsxs } from "react/jsx-runtime";
import { Check, Circle, Plus, Trash2 } from 'lucide-react';
import { useState } from 'react';
import { Button } from './ui/button';
import { Card } from './ui/card';
import { Input } from './ui/input';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue, } from './ui/select';
import { Badge } from './ui/badge';
import { ConfirmDialog } from './ConfirmDialog';
import { isOverdue } from '../lib/dates';
const PRIORITY_VARIANT = {
    low: 'secondary',
    medium: 'default',
    high: 'link',
    urgent: 'link',
};
export function TasksPage({ tasks, onCreate, onToggle, onDelete }) {
    const [title, setTitle] = useState('');
    const [priority, setPriority] = useState('medium');
    const [dueAt, setDueAt] = useState('');
    const [busy, setBusy] = useState(false);
    const [pendingDelete, setPendingDelete] = useState(null);
    const sorted = [...tasks].sort((a, b) => {
        const da = a.dueAt ? a.dueAt.slice(0, 10) : '9999-12-31';
        const db = b.dueAt ? b.dueAt.slice(0, 10) : '9999-12-31';
        return da.localeCompare(db);
    });
    async function submit(e) {
        e.preventDefault();
        const trimmed = title.trim();
        if (!trimmed || busy)
            return;
        setBusy(true);
        try {
            await onCreate({ title: trimmed, priority, dueAt: dueAt || null });
            setTitle('');
            setDueAt('');
        }
        finally {
            setBusy(false);
        }
    }
    return (_jsxs("div", { children: [_jsx("h1", { children: "Tasks" }), _jsx("p", { className: "sub", children: "Complete a task to earn XP. The highest priority first." }), _jsx(Card, { children: _jsxs("form", { onSubmit: submit, className: "flex flex-wrap items-center gap-2", children: [_jsx(Input, { value: title, onChange: (e) => setTitle(e.target.value), placeholder: "Add a task\u2026", maxLength: 200, "aria-label": "Task title", className: "min-w-48 flex-1" }), _jsxs(Select, { value: priority, onValueChange: (v) => setPriority(v), children: [_jsx(SelectTrigger, { "aria-label": "Priority", className: "w-36", children: _jsx(SelectValue, {}) }), _jsxs(SelectContent, { children: [_jsx(SelectItem, { value: "low", children: "Low" }), _jsx(SelectItem, { value: "medium", children: "Medium" }), _jsx(SelectItem, { value: "high", children: "High" }), _jsx(SelectItem, { value: "urgent", children: "Urgent" })] })] }), _jsx(Input, { type: "date", value: dueAt, onChange: (e) => setDueAt(e.target.value), "aria-label": "Due date", className: "w-44" }), _jsxs(Button, { type: "submit", disabled: !title.trim() || busy, children: [_jsx(Plus, { className: "h-4 w-4", "aria-hidden": true }), " Add"] })] }) }), _jsx("div", { style: { display: 'flex', flexDirection: 'column', gap: '0.25rem', marginTop: '1rem' }, children: sorted.length === 0 ? (_jsx("p", { style: { padding: '2rem 0', textAlign: 'center', color: 'var(--color-muted)' }, children: "No tasks yet. Add your first one above." })) : (sorted.map((task) => (_jsxs(Card, { className: "row", children: [_jsx("button", { type: "button", onClick: () => onToggle(task), "aria-pressed": task.status === 'done', "aria-label": task.status === 'done' ? `Reopen "${task.title}"` : `Complete "${task.title}"`, className: "text-muted hover:text-success", children: task.status === 'done' ? (_jsx(Check, { className: "h-5 w-5 text-success" })) : (_jsx(Circle, { className: "h-5 w-5" })) }), _jsxs("div", { className: "grow", style: { minWidth: 0 }, children: [_jsx("b", { style: { textDecoration: task.status === 'done' ? 'line-through' : undefined }, children: task.title }), task.dueAt !== null && (_jsxs("span", { className: "small", style: isOverdue(task.dueAt) ? { color: 'var(--color-destructive)' } : undefined, children: [isOverdue(task.dueAt) ? 'Overdue · ' : 'Due ', task.dueAt.slice(0, 10)] }))] }), _jsx(Badge, { variant: PRIORITY_VARIANT[task.priority], children: task.priority }), _jsx(Button, { variant: "outline", size: "sm", onClick: () => setPendingDelete(task), "aria-label": `Delete "${task.title}"`, children: _jsx(Trash2, { "aria-hidden": true }) })] }, task.id)))) }), pendingDelete && (_jsx(ConfirmDialog, { title: `Delete "${pendingDelete.title}"?`, body: "This task will be permanently deleted. This cannot be undone.", onCancel: () => setPendingDelete(null), onConfirm: () => {
                    onDelete(pendingDelete);
                    setPendingDelete(null);
                } }))] }));
}
