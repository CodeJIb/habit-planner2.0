import { jsx as _jsx, jsxs as _jsxs } from "react/jsx-runtime";
import { Check, Circle, Plus, Trash2 } from 'lucide-react';
import { useState } from 'react';
import { Badge } from './ui/badge';
import { Button } from './ui/button';
import { Card } from './ui/card';
import { Input } from './ui/input';
import { Progress } from './ui/progress';
import { ConfirmDialog } from './ConfirmDialog';
import { formatDue, isOverdue } from '../lib/dates';
const MUTED_SMALL = {
    display: 'block',
    fontSize: '0.75rem',
    color: 'var(--color-muted)',
};
/** Goals with milestones, target dates and completion XP. */
export function GoalsPage({ goals, onCreate, onSetStatus, onDelete, onAddMilestone, onToggleMilestone, onDeleteMilestone, }) {
    const [title, setTitle] = useState('');
    const [targetDate, setTargetDate] = useState('');
    const [busy, setBusy] = useState(false);
    const [pendingDelete, setPendingDelete] = useState(null);
    async function submit(e) {
        e.preventDefault();
        const trimmed = title.trim();
        if (!trimmed || busy)
            return;
        setBusy(true);
        try {
            await onCreate({ title: trimmed, targetDate: targetDate || null });
            setTitle('');
            setTargetDate('');
        }
        finally {
            setBusy(false);
        }
    }
    return (_jsxs("div", { className: "screen", children: [_jsx("h1", { children: "Goals" }), _jsx("p", { className: "sub", children: "Break big goals into milestones. Finish a goal for bonus XP." }), _jsx(Card, { children: _jsxs("form", { onSubmit: submit, className: "flex flex-wrap items-center gap-2", children: [_jsx(Input, { value: title, onChange: (e) => setTitle(e.target.value), placeholder: "Add a goal\u2026", maxLength: 200, "aria-label": "Goal title", className: "min-w-48 flex-1" }), _jsx(Input, { type: "date", value: targetDate, onChange: (e) => setTargetDate(e.target.value), "aria-label": "Target date", className: "w-44" }), _jsxs(Button, { type: "submit", disabled: !title.trim() || busy, children: [_jsx(Plus, { className: "h-4 w-4", "aria-hidden": true }), " Add"] })] }) }), _jsx("div", { style: { display: 'flex', flexDirection: 'column', gap: '0.75rem', marginTop: '1rem' }, children: goals.length === 0 ? (_jsx("p", { style: { padding: '2rem 0', textAlign: 'center', color: 'var(--color-muted)' }, children: "No goals yet. Add your first one above." })) : (goals.map((goal) => (_jsx(GoalCard, { goal: goal, onSetStatus: onSetStatus, onAddMilestone: onAddMilestone, onToggleMilestone: onToggleMilestone, onDeleteMilestone: onDeleteMilestone, onAskDelete: () => setPendingDelete(goal) }, goal.id)))) }), pendingDelete && (_jsx(ConfirmDialog, { title: `Delete "${pendingDelete.title}"?`, body: "This goal and its milestones will be permanently deleted. This cannot be undone.", onCancel: () => setPendingDelete(null), onConfirm: () => {
                    onDelete(pendingDelete);
                    setPendingDelete(null);
                } }))] }));
}
function GoalCard({ goal, onSetStatus, onAddMilestone, onToggleMilestone, onDeleteMilestone, onAskDelete, }) {
    const [milestoneTitle, setMilestoneTitle] = useState('');
    const [pendingDelete, setPendingDelete] = useState(null);
    const completed = goal.status === 'completed';
    const total = goal.milestones.length;
    const done = goal.milestones.filter((m) => m.status === 'completed').length;
    const pct = total > 0 ? Math.round((done / total) * 100) : 0;
    const overdue = !!goal.targetDate && !completed && isOverdue(goal.targetDate);
    async function addMilestone(e) {
        e.preventDefault();
        const trimmed = milestoneTitle.trim();
        if (!trimmed)
            return;
        await onAddMilestone(goal, { title: trimmed });
        setMilestoneTitle('');
    }
    return (_jsxs(Card, { children: [_jsxs("div", { style: { display: 'flex', alignItems: 'center', gap: '0.75rem' }, children: [_jsx("button", { type: "button", onClick: () => onSetStatus(goal, completed ? 'active' : 'completed'), "aria-pressed": completed, "aria-label": completed ? `Reopen goal "${goal.title}"` : `Complete goal "${goal.title}"`, className: "text-muted hover:text-success", children: completed ? (_jsx(Check, { className: "h-5 w-5 text-success" })) : (_jsx(Circle, { className: "h-5 w-5" })) }), _jsxs("div", { style: { minWidth: 0, flex: 1 }, children: [_jsx("b", { style: {
                                    textDecoration: completed ? 'line-through' : undefined,
                                    display: 'block',
                                }, children: goal.title }), _jsx("span", { style: {
                                    ...MUTED_SMALL,
                                    color: overdue ? 'var(--color-destructive)' : undefined,
                                }, children: goal.targetDate
                                    ? `${overdue ? 'Overdue · ' : 'Target '}${formatDue(goal.targetDate)}`
                                    : 'No target date' })] }), completed && _jsx(Badge, { variant: "secondary", children: "Done" }), total > 0 && (_jsxs("span", { className: "xp-pill", "aria-label": `${done} of ${total} milestones`, children: [done, "/", total] })), _jsx(Button, { variant: "outline", size: "sm", onClick: onAskDelete, "aria-label": `Delete goal "${goal.title}"`, children: _jsx(Trash2, { "aria-hidden": true }) })] }), total > 0 && (_jsx("div", { style: { marginTop: '0.75rem' }, children: _jsx(Progress, { value: pct, "aria-label": `${goal.title} milestone progress` }) })), total > 0 && (_jsx("div", { style: { display: 'flex', flexDirection: 'column', gap: '0.25rem' }, children: goal.milestones.map((milestone) => {
                    const isDone = milestone.status === 'completed';
                    return (_jsxs("div", { style: { display: 'flex', alignItems: 'center', gap: '0.5rem' }, children: [_jsx("button", { type: "button", onClick: () => onToggleMilestone(milestone), "aria-pressed": isDone, "aria-label": isDone ? `Reopen milestone "${milestone.title}"` : `Complete milestone "${milestone.title}"`, className: "text-muted hover:text-success", children: isDone ? (_jsx(Check, { className: "h-4 w-4 text-success" })) : (_jsx(Circle, { className: "h-4 w-4" })) }), _jsx("span", { style: {
                                    flex: 1,
                                    minWidth: 0,
                                    textDecoration: isDone ? 'line-through' : undefined,
                                    ...MUTED_SMALL,
                                    display: 'inline-block',
                                    fontSize: '0.875rem',
                                }, children: milestone.title }), _jsx("button", { type: "button", onClick: () => setPendingDelete(milestone), "aria-label": `Delete milestone "${milestone.title}"`, className: "text-muted hover:text-destructive", children: _jsx(Trash2, { className: "h-4 w-4", "aria-hidden": true }) })] }, milestone.id));
                }) })), _jsxs("form", { onSubmit: addMilestone, className: "flex gap-2", style: { marginTop: '0.75rem' }, children: [_jsx(Input, { value: milestoneTitle, onChange: (e) => setMilestoneTitle(e.target.value), placeholder: "Add a milestone\u2026", maxLength: 200, "aria-label": "Milestone title", className: "min-w-0 flex-1" }), _jsxs(Button, { type: "submit", variant: "outline", disabled: !milestoneTitle.trim(), children: [_jsx(Plus, { className: "h-4 w-4", "aria-hidden": true }), " Add"] })] }), pendingDelete && (_jsx(ConfirmDialog, { title: `Delete "${pendingDelete.title}"?`, body: "This milestone will be permanently deleted. This cannot be undone.", onCancel: () => setPendingDelete(null), onConfirm: () => {
                    onDeleteMilestone(pendingDelete);
                    setPendingDelete(null);
                } }))] }));
}
