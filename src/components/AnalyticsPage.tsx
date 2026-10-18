import { CheckCheck, Flame, RefreshCw, Sparkles, Target, Timer, TrendingUp } from 'lucide-react'
import { Fragment } from 'react'
import { Badge } from './ui/badge'
import { Button } from './ui/button'
import { Card } from './ui/card'
import { Progress } from './ui/progress'
import type { Analytics, AnalyticsSummary, PlayerProfile, XpPoint } from '../types'
import { addDays, localToday, mondayOf } from '../lib/dates'

interface Props {
  data: Analytics | null
  profile: PlayerProfile
  onRefresh: () => void
}

const BAR_DAYS = 14
const HEAT_WEEKS = 8
const AXIS = 'var(--color-muted)'
const EMPTY_STYLE = {
  padding: '2rem 0',
  textAlign: 'center',
  color: 'var(--color-muted)',
} as const

/** Total XP needed to reach `level`, mirroring xp_for_level in services/gamification.rs. */
function xpForLevel(level: number): number {
  const n = Math.max(0, level - 1)
  return 100 * n * n
}

/** A dense window always reads as a timeline, so days without data stay as gaps. */
function lastNDays(count: number, rows: { date: string; value: number }[]): { date: string; value: number }[] {
  const values = new Map<string, number>()
  for (const row of rows) values.set(row.date, row.value)

  const today = localToday()
  const days: { date: string; value: number }[] = []
  for (let offset = count - 1; offset >= 0; offset--) {
    const date = addDays(today, -offset)
    days.push({ date, value: values.get(date) ?? 0 })
  }
  return days
}

/** Cumulative XP with a dashed line at the XP every level starts at. */
function XpChart({ points }: { points: XpPoint[] }) {
  if (points.length === 0) {
    return <p style={EMPTY_STYLE}>No XP earned yet. Finish a task or check in a habit to start the line.</p>
  }

  const width = 640
  const height = 220
  const pad = { top: 18, right: 12, bottom: 26, left: 12 }
  const innerW = width - pad.left - pad.right
  const innerH = height - pad.top - pad.bottom
  const peak = Math.max(points[points.length - 1].cumulative, 1)

  const x = (index: number) =>
    pad.left + (points.length === 1 ? innerW / 2 : (index / (points.length - 1)) * innerW)
  const y = (value: number) => pad.top + innerH - (value / peak) * innerH

  const levels: number[] = []
  for (let level = 2; xpForLevel(level) <= peak; level++) levels.push(level)

  const line = points.map((point, index) => `${x(index)},${y(point.cumulative)}`).join(' ')
  const area = `${pad.left},${pad.top + innerH} ${line} ${x(points.length - 1)},${pad.top + innerH}`

  return (
    <svg
      viewBox={`0 0 ${width} ${height}`}
      style={{ width: '100%', height: 'auto', display: 'block' }}
      role="img"
      aria-label={`Cumulative XP from ${points[0].date} to ${points[points.length - 1].date}`}
    >
      {levels.map((level) => (
        <g key={level}>
          <line
            x1={pad.left}
            x2={width - pad.right}
            y1={y(xpForLevel(level))}
            y2={y(xpForLevel(level))}
            stroke={AXIS}
            strokeWidth="1"
            strokeDasharray="4 4"
            opacity="0.6"
          />
          <text x={pad.left + 4} y={y(xpForLevel(level)) - 5} fill={AXIS} fontSize="10">
            Lv {level}
          </text>
        </g>
      ))}

      {points.length > 1 && <polygon points={area} fill="var(--color-primary)" opacity="0.12" />}
      {points.length > 1 && (
        <polyline
          points={line}
          fill="none"
          stroke="var(--color-primary)"
          strokeWidth="2"
          strokeLinecap="round"
          strokeLinejoin="round"
        />
      )}
      {points.map((point, index) => (
        <circle key={point.date} cx={x(index)} cy={y(point.cumulative)} r="2.5" fill="var(--color-primary)">
          <title>{`${point.date}: +${point.amount} XP, total ${point.cumulative}`}</title>
        </circle>
      ))}

      <line
        x1={pad.left}
        x2={width - pad.right}
        y1={pad.top + innerH}
        y2={pad.top + innerH}
        stroke={AXIS}
        opacity="0.4"
      />
      <text x={pad.left} y={12} fill={AXIS} fontSize="10">
        {peak} XP
      </text>
      <text x={pad.left} y={height - 8} fill={AXIS} fontSize="10">
        {points[0].date.slice(5)}
      </text>
      <text x={width - pad.right} y={height - 8} fill={AXIS} fontSize="10" textAnchor="end">
        {points[points.length - 1].date.slice(5)}
      </text>
    </svg>
  )
}

/** One bar per day, with the value on hover. */
function DayBars({ items, unit }: { items: { date: string; value: number }[]; unit: string }) {
  const total = items.reduce((sum, item) => sum + item.value, 0)
  if (total === 0) {
    return <p style={EMPTY_STYLE}>Nothing logged in this window yet.</p>
  }

  const width = 640
  const height = 150
  const pad = { top: 16, right: 8, bottom: 24, left: 8 }
  const innerW = width - pad.left - pad.right
  const innerH = height - pad.top - pad.bottom
  const peak = Math.max(...items.map((item) => item.value), 1)
  const step = innerW / items.length
  const barW = Math.max(3, step - 6)

  return (
    <svg
      viewBox={`0 0 ${width} ${height}`}
      style={{ width: '100%', height: 'auto', display: 'block' }}
      role="img"
      aria-label={`${unit} per day over the last ${items.length} days`}
    >
      <line
        x1={pad.left}
        x2={width - pad.right}
        y1={pad.top + innerH}
        y2={pad.top + innerH}
        stroke={AXIS}
        opacity="0.4"
      />
      {items.map((item, index) => {
        const barH = (item.value / peak) * innerH
        const left = pad.left + index * step + (step - barW) / 2
        return (
          <g key={item.date}>
            <rect
              x={left}
              y={pad.top + innerH - barH}
              width={barW}
              height={Math.max(barH, 2)}
              rx="2"
              fill="var(--color-primary)"
              opacity={item.value > 0 ? 1 : 0.18}
            >
              <title>{`${item.date}: ${item.value} ${unit}`}</title>
            </rect>
            {index % 2 === 0 && (
              <text x={left + barW / 2} y={height - 8} fill={AXIS} fontSize="9" textAnchor="middle">
                {item.date.slice(5)}
              </text>
            )}
          </g>
        )
      })}
      <text x={pad.left} y={10} fill={AXIS} fontSize="10">
        max {peak}
      </text>
    </svg>
  )
}

/** Days of the week (rows) by weeks (columns); a filled cell had at least one check-in. */
function HeatMap({ checkins }: { checkins: string[] }) {
  const done = new Set(checkins)
  const today = localToday()
  const start = addDays(mondayOf(today), -(HEAT_WEEKS - 1) * 7)
  const labels = ['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun']

  return (
    <div role="img" aria-label="Habit check-ins by day over the last eight weeks">
      <div
        style={{
          display: 'grid',
          gridTemplateColumns: `2.25rem repeat(${HEAT_WEEKS}, minmax(0, 1fr))`,
          gap: '0.25rem',
          maxWidth: '28rem',
          alignItems: 'center',
        }}
      >
        {labels.map((label, day) => (
          <Fragment key={label}>
            <span className="small" style={{ fontSize: '0.65rem', lineHeight: '1.1rem' }}>
              {label}
            </span>
            {Array.from({ length: HEAT_WEEKS }, (_, week) => {
              const date = addDays(start, week * 7 + day)
              const future = date > today
              const checked = done.has(date)
              return (
                <span
                  key={date}
                  title={future ? undefined : `${date}${checked ? ' · checked in' : ''}`}
                  style={{
                    height: '1.1rem',
                    borderRadius: '0.25rem',
                    background: checked ? 'var(--color-ok)' : 'var(--color-accent)',
                    opacity: future ? 0.15 : checked ? 1 : 0.4,
                  }}
                />
              )
            })}
          </Fragment>
        ))}
      </div>
      <div
        style={{
          display: 'flex',
          alignItems: 'center',
          gap: '0.375rem',
          marginTop: '0.75rem',
          fontSize: '0.75rem',
          color: 'var(--color-muted)',
        }}
      >
        Less
        <span
          style={{
            width: '0.9rem',
            height: '0.9rem',
            borderRadius: '0.2rem',
            background: 'var(--color-accent)',
            opacity: 0.4,
          }}
        />
        <span
          style={{
            width: '0.9rem',
            height: '0.9rem',
            borderRadius: '0.2rem',
            background: 'var(--color-ok)',
          }}
        />
        More
      </div>
    </div>
  )
}

function SummaryCards({ summary, profile }: { summary: AnalyticsSummary; profile: PlayerProfile }) {
  const cards = [
    { icon: <TrendingUp aria-hidden />, label: 'Current level', value: String(summary.level) },
    { icon: <Sparkles aria-hidden />, label: 'Total XP', value: String(summary.totalXp) },
    { icon: <Flame aria-hidden />, label: 'Longest streak', value: `${summary.longestStreak} d` },
    { icon: <CheckCheck aria-hidden />, label: 'Tasks completed', value: String(summary.tasksCompleted) },
    { icon: <Timer aria-hidden />, label: 'Focus minutes', value: String(summary.focusMinutes) },
    { icon: <Target aria-hidden />, label: 'Goals completed', value: String(summary.goalsCompleted) },
  ]
  const pct = Math.min(100, Math.round((profile.xpIntoLevel / profile.xpForNext) * 100))

  return (
    <>
      <div
        style={{
          display: 'grid',
          gridTemplateColumns: 'repeat(auto-fill, minmax(10.5rem, 1fr))',
          gap: '0.75rem',
        }}
      >
        {cards.map((card) => (
          <Card key={card.label} style={{ gap: '0.5rem', padding: '0.875rem 1rem' }}>
            <div
              style={{
                display: 'flex',
                alignItems: 'center',
                gap: '0.5rem',
                color: 'var(--color-muted)',
              }}
            >
              {card.icon}
              <span style={{ fontSize: '0.75rem' }}>{card.label}</span>
            </div>
            <b style={{ fontSize: '1.5rem', lineHeight: 1 }}>{card.value}</b>
          </Card>
        ))}
      </div>

      <Card aria-label="Level progress" style={{ gap: '0.5rem', padding: '0.875rem 1rem' }}>
        <div style={{ display: 'flex', alignItems: 'baseline', justifyContent: 'space-between', gap: '0.75rem' }}>
          <b>Level {profile.level}</b>
          <span className="small">
            {profile.xpIntoLevel} of {profile.xpForNext} XP to level {profile.level + 1}
          </span>
        </div>
        <Progress value={pct} aria-label="XP progress to next level" />
      </Card>
    </>
  )
}

/** Charts and totals derived from tasks, habits, focus sessions and the XP ledger. */
export function AnalyticsPage({ data, profile, onRefresh }: Props) {
  const taskBars = lastNDays(
    BAR_DAYS,
    (data?.taskCompletions ?? []).map((row) => ({ date: row.date, value: row.count })),
  )
  const focusBars = lastNDays(
    BAR_DAYS,
    (data?.focusMinutes ?? []).map((row) => ({ date: row.date, value: row.minutes })),
  )

  return (
    <div className="screen">
      <div style={{ display: 'flex', alignItems: 'flex-start', gap: '0.75rem' }}>
        <div className="grow">
          <h1>Analytics</h1>
          <p className="sub">Your XP, focus time and habit history in one place.</p>
        </div>
        <Button variant="outline" size="sm" onClick={onRefresh} disabled={!data}>
          <RefreshCw aria-hidden /> Refresh
        </Button>
      </div>

      {!data ? (
        <p style={{ color: 'var(--color-muted)' }}>Loading analytics…</p>
      ) : (
        <>
          <SummaryCards summary={data.summary} profile={profile} />

          <h2 style={{ marginTop: '1.5rem', display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
            XP progression <Badge variant="secondary">Level {data.summary.level}</Badge>
          </h2>
          <Card>
            <span className="small">
              Cumulative XP per day, with the XP every level starts at marked as a dashed line.
            </span>
            <XpChart points={data.xp} />
          </Card>

          <h2 style={{ marginTop: '1.5rem', display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
            Tasks completed <Badge variant="outline">Last {BAR_DAYS} days</Badge>
          </h2>
          <Card>
            <DayBars items={taskBars} unit="tasks" />
          </Card>

          <h2 style={{ marginTop: '1.5rem', display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
            Focus time <Badge variant="outline">Last {BAR_DAYS} days</Badge>
          </h2>
          <Card>
            <DayBars items={focusBars} unit="minutes" />
          </Card>

          <h2 style={{ marginTop: '1.5rem', display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
            Habit consistency <Badge variant="outline">Last {HEAT_WEEKS} weeks</Badge>
          </h2>
          <Card>
            <HeatMap checkins={data.habitCheckins} />
          </Card>
        </>
      )}
    </div>
  )
}
