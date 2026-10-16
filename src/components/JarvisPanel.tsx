import { Mic } from 'lucide-react'
import { Button } from './ui/button'

interface Props {
  /** Messages sent to the assistant today. Currently always empty: the AI
   *  service is a stub (Phase 5) and the panel is a placeholder. */
  messages: string[]
}

const SUGGESTIONS = ['Plan my day', 'Check my streaks', 'How much XP to level up?']

/** Right-hand assistant panel. Phase 5 wires this up to the AI service. */
export function JarvisPanel({ messages }: Props) {
  return (
    <aside className="assistant" aria-label="JARVIS assistant">
      <header>
        <div className="orb" aria-hidden />
        <div>
          <b>JARVIS</b>
          <span>Assistant · arrives in Phase 5</span>
        </div>
      </header>

      <div className="log" aria-live="polite">
        {messages.length === 0 ? (
          <p style={{ color: 'var(--color-muted)' }}>Hello. Ask me to plan your day or check your progress.</p>
        ) : (
          messages.map((message, index) => (
            <p key={index} className="msg assistant">
              {message}
            </p>
          ))
        )}
      </div>

      <footer>
        <div className="chips" style={{ display: 'flex', gap: '0.375rem', flexWrap: 'wrap' }}>
          {SUGGESTIONS.map((suggestion) => (
            <Button key={suggestion} variant="outline" size="sm" disabled>
              {suggestion}
            </Button>
          ))}
        </div>
        <form onSubmit={(e) => e.preventDefault()}>
          <input
            placeholder="Ask JARVIS…"
            aria-label="Message JARVIS"
            disabled
          />
          <Button type="button" variant="outline" size="icon" aria-label="Voice input (coming soon)" disabled>
            <Mic aria-hidden />
          </Button>
          <Button type="button" disabled>
            Send
          </Button>
        </form>
        <p style={{ fontSize: '0.75rem', color: 'var(--color-muted)' }}>
          The AI service is not available yet. Everything here is a preview.
        </p>
      </footer>
    </aside>
  )
}
