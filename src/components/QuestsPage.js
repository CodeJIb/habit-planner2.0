import { jsx as _jsx, jsxs as _jsxs } from "react/jsx-runtime";
import { Check, Flame, Star, Target, Trophy } from 'lucide-react';
import { Badge } from './ui/badge';
import { Card } from './ui/card';
import { Progress } from './ui/progress';
/** Daily and weekly quests, plus the achievement grid. */
export function QuestsPage({ quests, achievements }) {
    const daily = quests.filter((q) => q.questType === 'daily');
    const weekly = quests.filter((q) => q.questType === 'weekly');
    const unlocked = achievements.filter((a) => a.unlocked).length;
    return (_jsxs("div", { className: "screen", children: [_jsx("h1", { children: "Quests" }), _jsx("p", { className: "sub", children: "Complete tasks and habits to finish quests and earn bonus XP." }), _jsx(QuestSection, { title: "Daily quests", quests: daily }), _jsx(QuestSection, { title: "Weekly quests", quests: weekly }), _jsxs("section", { "aria-label": "Achievements", children: [_jsxs("h2", { children: ["Achievements", ' ', _jsxs("span", { className: "text-muted", style: { fontSize: '0.875rem', fontWeight: 400 }, children: ["(", unlocked, "/", achievements.length, " unlocked)"] })] }), _jsx("div", { style: {
                            display: 'grid',
                            gridTemplateColumns: 'repeat(auto-fill, minmax(15rem, 1fr))',
                            gap: '0.75rem',
                        }, children: achievements.map((achievement) => (_jsx(AchievementCard, { achievement: achievement }, achievement.id))) })] })] }));
}
function QuestSection({ title, quests }) {
    return (_jsxs("section", { "aria-label": title, style: { marginBottom: '1.5rem' }, children: [_jsx("h2", { children: title }), _jsx("div", { style: { display: 'flex', flexDirection: 'column', gap: '0.25rem' }, children: quests.length === 0 ? (_jsxs("p", { style: { padding: '1.5rem 0', textAlign: 'center', color: 'var(--color-muted)' }, children: ["No ", title.toLowerCase(), " right now."] })) : (quests.map((quest) => _jsx(QuestRow, { quest: quest }, quest.id))) })] }));
}
function QuestRow({ quest }) {
    const pct = quest.targetValue > 0
        ? Math.min(100, Math.round((quest.currentValue / quest.targetValue) * 100))
        : 0;
    return (_jsxs(Card, { className: "row", "data-quest-row": quest.id, children: [_jsx(Target, { "aria-hidden": true, className: "text-xp" }), _jsxs("div", { className: "grow", children: [_jsxs("div", { style: { display: 'flex', alignItems: 'center', gap: '0.5rem' }, children: [_jsx("b", { children: quest.title }), quest.completed && _jsx(Badge, { variant: "secondary", children: "Done" })] }), _jsx("span", { className: "small", children: quest.description }), _jsx("div", { style: { marginTop: '0.5rem' }, children: _jsx(Progress, { value: pct, "aria-label": `${quest.title} progress` }) }), _jsxs("span", { className: "small", style: { marginTop: '0.25rem' }, children: [quest.currentValue, " / ", quest.targetValue] })] }), _jsxs("span", { className: "xp-pill", children: ["+", quest.xpReward, " XP"] })] }));
}
function AchievementCard({ achievement }) {
    return (_jsxs(Card, { "data-achievement-card": achievement.code, style: { opacity: achievement.unlocked ? 1 : 0.55 }, children: [_jsxs("div", { style: { display: 'flex', alignItems: 'center', gap: '0.625rem' }, children: [_jsx(AchievementIcon, { icon: achievement.icon, unlocked: achievement.unlocked }), _jsxs("div", { style: { minWidth: 0 }, children: [_jsx("b", { children: achievement.name }), _jsx("span", { style: {
                                    display: 'block',
                                    color: 'var(--color-muted)',
                                    fontSize: '0.75rem',
                                }, children: achievement.description })] })] }), _jsxs("div", { style: {
                    marginTop: '0.5rem',
                    display: 'flex',
                    justifyContent: 'space-between',
                    alignItems: 'center',
                }, children: [_jsxs("span", { className: "xp-pill", children: ["+", achievement.xpReward, " XP"] }), _jsx("span", { style: {
                            fontSize: '0.75rem',
                            color: achievement.unlocked ? 'var(--color-success)' : 'var(--color-muted)',
                        }, children: achievement.unlocked ? 'Unlocked' : 'Locked' })] })] }));
}
function AchievementIcon({ icon, unlocked }) {
    const className = unlocked ? 'text-xp' : 'text-muted';
    if (icon === 'flame')
        return _jsx(Flame, { "aria-hidden": true, className: className });
    if (icon === 'star')
        return _jsx(Star, { "aria-hidden": true, className: className });
    if (icon === 'trophy')
        return _jsx(Trophy, { "aria-hidden": true, className: className });
    return _jsx(Check, { "aria-hidden": true, className: className });
}
