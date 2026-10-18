import { Bell } from 'lucide-react'
import { useEffect, useState } from 'react'
import { api } from '../lib/api'
import type { NotificationSettings } from '../types'
import { Button } from './ui/button'
import { Card } from './ui/card'
import { Checkbox } from './ui/checkbox'
import { Input } from './ui/input'

interface Props {
  onClose: () => void
}

/**
 * Setup dialog for phone reminders: the user picks an ntfy topic, subscribes to
 * the same topic in the free ntfy phone app, and the app pushes alerts there.
 */
export function ReminderSettings({ onClose }: Props) {
  const [settings, setSettings] = useState<NotificationSettings | null>(null)
  const [enabled, setEnabled] = useState(false)
  const [topic, setTopic] = useState('')
  const [server, setServer] = useState('')
  const [busy, setBusy] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [notice, setNotice] = useState<string | null>(null)

  useEffect(() => {
    let cancelled = false
    void api
      .getNotificationSettings()
      .then((loaded) => {
        if (cancelled) return
        setSettings(loaded)
        setEnabled(loaded.enabled)
        setTopic(loaded.topic)
        setServer(loaded.server)
      })
      .catch((e: unknown) => {
        if (!cancelled) setError(e instanceof Error ? e.message : String(e))
      })
    return () => {
      cancelled = true
    }
  }, [])

  // Escape closes the dialog.
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape') onClose()
    }
    window.addEventListener('keydown', onKey)
    return () => window.removeEventListener('keydown', onKey)
  }, [onClose])

  const ready = settings !== null && !busy

  function asError(e: unknown): string {
    return e instanceof Error ? e.message : String(e)
  }

  async function save() {
    setBusy(true)
    setError(null)
    setNotice(null)
    try {
      const updated = await api.setNotificationSettings({ enabled, topic, server })
      setSettings(updated)
      setNotice(updated.enabled && updated.topic ? 'Saved.' : 'Saved — reminders are off.')
    } catch (e) {
      setError(asError(e))
    } finally {
      setBusy(false)
    }
  }

  async function sendTest() {
    setBusy(true)
    setError(null)
    setNotice(null)
    try {
      await api.sendTestNotification()
      setNotice('Test notification sent — check your phone.')
    } catch (e) {
      setError(asError(e))
    } finally {
      setBusy(false)
    }
  }

  async function checkNow() {
    setBusy(true)
    setError(null)
    setNotice(null)
    try {
      const result = await api.sendDueReminders()
      setNotice(
        result.sent === 0
          ? 'Nothing to remind you about right now.'
          : `Sent ${result.sent} reminder${result.sent === 1 ? '' : 's'}.`,
      )
    } catch (e) {
      setError(asError(e))
    } finally {
      setBusy(false)
    }
  }

  return (
    <div
      role="dialog"
      aria-modal="true"
      aria-label="Phone reminders"
      style={{
        position: 'fixed',
        inset: 0,
        zIndex: 40,
        display: 'grid',
        placeItems: 'center',
        background: 'rgba(0, 0, 0, 0.4)',
        padding: '1rem',
      }}
      onClick={onClose}
    >
      <Card
        style={{ maxWidth: '34rem', width: '100%', gap: '1rem', padding: '1.25rem' }}
        onClick={(e) => e.stopPropagation()}
      >
        <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
          <Bell aria-hidden />
          <h2 style={{ margin: 0 }}>Phone reminders</h2>
          <Button
            type="button"
            variant="ghost"
            size="sm"
            onClick={onClose}
            style={{ marginLeft: 'auto' }}
          >
            Close
          </Button>
        </div>

        <p style={{ margin: 0, color: 'var(--color-muted)', fontSize: '0.875rem' }}>
          Habit Planner pushes a notification to your phone when a task comes due, and sends one
          digest a day. It uses ntfy.sh, which is free and needs no account: install{' '}
          <b>ntfy</b> (Android/iOS), subscribe to a topic, and put the same name below. Treat the
          topic like a password — anyone who knows it can subscribe too.
        </p>

        <label style={{ display: 'grid', gap: '0.375rem' }}>
          <span style={{ fontSize: '0.875rem', fontWeight: 600 }}>Topic name</span>
          <Input
            value={topic}
            onChange={(e) => setTopic(e.target.value)}
            placeholder="my-habit-reminders"
            aria-label="ntfy topic"
            disabled={!ready}
            autoComplete="off"
          />
        </label>

        <label style={{ display: 'grid', gap: '0.375rem' }}>
          <span style={{ fontSize: '0.875rem', fontWeight: 600 }}>
            Server <span style={{ color: 'var(--color-muted)' }}>(optional)</span>
          </span>
          <Input
            value={server}
            onChange={(e) => setServer(e.target.value)}
            placeholder="https://ntfy.sh"
            aria-label="ntfy server"
            disabled={!ready}
            autoComplete="off"
          />
        </label>

        <label
          style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', fontSize: '0.875rem' }}
        >
          <Checkbox
            id="notify-enabled"
            checked={enabled}
            onCheckedChange={(value) => setEnabled(value === true)}
            disabled={!ready}
          />
          <span style={{ fontWeight: 600 }}>Enable phone reminders</span>
        </label>

        <div style={{ display: 'flex', gap: '0.5rem', flexWrap: 'wrap' }}>
          <Button type="button" onClick={save} disabled={!ready}>
            Save
          </Button>
          <Button type="button" variant="outline" onClick={sendTest} disabled={!ready}>
            Send test
          </Button>
          <Button type="button" variant="outline" onClick={checkNow} disabled={!ready}>
            Check now
          </Button>
        </div>

        {error && (
          <p role="alert" style={{ margin: 0, color: 'var(--color-destructive)', fontSize: '0.875rem' }}>
            {error}
          </p>
        )}
        {notice && (
          <p role="status" style={{ margin: 0, color: 'var(--color-muted)', fontSize: '0.875rem' }}>
            {notice}
          </p>
        )}
      </Card>
    </div>
  )
}
