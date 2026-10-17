import { Check, Flame, Star, Target, Trophy } from 'lucide-react'
import { Badge } from './ui/badge'
import { Card } from './ui/card'
import { Progress } from './ui/progress'
import type { Achievement, Quest } from '../types'

interface Props {
  quests: Quest[]
  achievements: Achievement[]
}

/** Daily and weekly quests, plus the achievement grid. */
export function QuestsPage({ quests, achievements }: Props) {
  const daily = quests.filter((q) => q.questType === 'daily')
  const weekly = quests.filter((q) => q.questType === 'weekly')
  const unlocked = achievements.filter((a) => a.unlocked).length

  return (
    <div className="screen">
      <h1>Quests</h1>
      <p className="sub">Complete tasks and habits to finish quests and earn bonus XP.</p>

      <QuestSection title="Daily quests" quests={daily} />
      <QuestSection title="Weekly quests" quests={weekly} />

      <section aria-label="Achievements">
        <h2>
          Achievements{' '}
          <span className="text-muted" style={{ fontSize: '0.875rem', fontWeight: 400 }}>
            ({unlocked}/{achievements.length} unlocked)
          </span>
        </h2>
        <div
          style={{
            display: 'grid',
            gridTemplateColumns: 'repeat(auto-fill, minmax(15rem, 1fr))',
            gap: '0.75rem',
          }}
        >
          {achievements.map((achievement) => (
            <AchievementCard key={achievement.id} achievement={achievement} />
          ))}
        </div>
      </section>
    </div>
  )
}

function QuestSection({ title, quests }: { title: string; quests: Quest[] }) {
  return (
    <section aria-label={title} style={{ marginBottom: '1.5rem' }}>
      <h2>{title}</h2>
      <div style={{ display: 'flex', flexDirection: 'column', gap: '0.25rem' }}>
        {quests.length === 0 ? (
          <p style={{ padding: '1.5rem 0', textAlign: 'center', color: 'var(--color-muted)' }}>
            No {title.toLowerCase()} right now.
          </p>
        ) : (
          quests.map((quest) => <QuestRow key={quest.id} quest={quest} />)
        )}
      </div>
    </section>
  )
}

function QuestRow({ quest }: { quest: Quest }) {
  const pct =
    quest.targetValue > 0
      ? Math.min(100, Math.round((quest.currentValue / quest.targetValue) * 100))
      : 0

  return (
    <Card className="row" data-quest-row={quest.id}>
      <Target aria-hidden className="text-xp" />
      <div className="grow">
        <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
          <b>{quest.title}</b>
          {quest.completed && <Badge variant="secondary">Done</Badge>}
        </div>
        <span className="small">{quest.description}</span>
        <div style={{ marginTop: '0.5rem' }}>
          <Progress value={pct} aria-label={`${quest.title} progress`} />
        </div>
        <span className="small" style={{ marginTop: '0.25rem' }}>
          {quest.currentValue} / {quest.targetValue}
        </span>
      </div>
      <span className="xp-pill">+{quest.xpReward} XP</span>
    </Card>
  )
}

function AchievementCard({ achievement }: { achievement: Achievement }) {
  return (
    <Card
      data-achievement-card={achievement.code}
      style={{ opacity: achievement.unlocked ? 1 : 0.55 }}
    >
      <div style={{ display: 'flex', alignItems: 'center', gap: '0.625rem' }}>
        <AchievementIcon icon={achievement.icon} unlocked={achievement.unlocked} />
        <div style={{ minWidth: 0 }}>
          <b>{achievement.name}</b>
          <span
            style={{
              display: 'block',
              color: 'var(--color-muted)',
              fontSize: '0.75rem',
            }}
          >
            {achievement.description}
          </span>
        </div>
      </div>
      <div
        style={{
          marginTop: '0.5rem',
          display: 'flex',
          justifyContent: 'space-between',
          alignItems: 'center',
        }}
      >
        <span className="xp-pill">+{achievement.xpReward} XP</span>
        <span
          style={{
            fontSize: '0.75rem',
            color: achievement.unlocked ? 'var(--color-success)' : 'var(--color-muted)',
          }}
        >
          {achievement.unlocked ? 'Unlocked' : 'Locked'}
        </span>
      </div>
    </Card>
  )
}

function AchievementIcon({ icon, unlocked }: { icon: string | null; unlocked: boolean }) {
  const className = unlocked ? 'text-xp' : 'text-muted'
  if (icon === 'flame') return <Flame aria-hidden className={className} />
  if (icon === 'star') return <Star aria-hidden className={className} />
  if (icon === 'trophy') return <Trophy aria-hidden className={className} />
  return <Check aria-hidden className={className} />
}
