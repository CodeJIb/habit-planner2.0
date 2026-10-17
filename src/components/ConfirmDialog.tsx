import { useEffect } from 'react'
import { Button } from './ui/button'
import { Card } from './ui/card'

interface Props {
  title: string
  body: string
  confirmLabel?: string
  onConfirm: () => void
  onCancel: () => void
}

/** Small confirm dialog used before a destructive action. */
export function ConfirmDialog({ title, body, confirmLabel = 'Delete', onConfirm, onCancel }: Props) {
  // Escape cancels the dialog.
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape') onCancel()
    }
    window.addEventListener('keydown', onKey)
    return () => window.removeEventListener('keydown', onKey)
  }, [onCancel])

  return (
    <div
      role="dialog"
      aria-modal="true"
      aria-label={title}
      style={{
        position: 'fixed',
        inset: 0,
        zIndex: 40,
        display: 'grid',
        placeItems: 'center',
        background: 'rgba(0, 0, 0, 0.4)',
      }}
      onClick={onCancel}
    >
      <Card
        style={{ maxWidth: '26rem', width: '90vw' }}
        onClick={(e) => e.stopPropagation()}
      >
        <h2 style={{ marginTop: 0 }}>{title}</h2>
        <p style={{ color: 'var(--color-muted)' }}>{body}</p>
        <div style={{ display: 'flex', gap: '0.5rem', justifyContent: 'flex-end' }}>
          <Button variant="outline" onClick={onCancel} autoFocus>
            Cancel
          </Button>
          <Button variant="destructive" onClick={onConfirm}>
            {confirmLabel}
          </Button>
        </div>
      </Card>
    </div>
  )
}
