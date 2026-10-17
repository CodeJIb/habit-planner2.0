import { Mic, Plus, Settings, Trash2 } from 'lucide-react'
import { useEffect, useRef, useState } from 'react'
import { Button } from './ui/button'
import { Input } from './ui/input'
import { ConfirmDialog } from './ConfirmDialog'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from './ui/select'
import type { AiConversation, AiMessage, AiSettings, AiSettingsInput } from '../types'

interface Props {
  messages: AiMessage[]
  conversations: AiConversation[]
  activeConversationId: number | null
  /** True while a reply is on the way. */
  busy: boolean
  /** Assistant text streaming in for the in-flight turn. */
  streaming: string
  /** The user's line, shown immediately while the reply is on the way. */
  pendingUser: string | null
  settings: AiSettings | null
  onSend: (text: string) => void
  onSelectConversation: (id: number) => void
  onNewConversation: () => void
  onDeleteConversation: (id: number) => void
  onSaveSettings: (input: AiSettingsInput) => void
}

const SUGGESTIONS = ['Plan my day', 'Check my streaks', 'How much XP to level up?']

/** Right-hand assistant panel. Talks to the Rust AI service (Phase 5). */
export function JarvisPanel({
  messages,
  conversations,
  activeConversationId,
  busy,
  streaming,
  pendingUser,
  settings,
  onSend,
  onSelectConversation,
  onNewConversation,
  onDeleteConversation,
  onSaveSettings,
}: Props) {
  const [draft, setDraft] = useState('')
  const [showSettings, setShowSettings] = useState(false)
  const [apiKey, setApiKey] = useState('')
  /** Local overrides while editing; each falls back to the saved value. */
  const [modelDraft, setModelDraft] = useState<string | null>(null)
  const [baseUrlDraft, setBaseUrlDraft] = useState<string | null>(null)
  const [pendingDelete, setPendingDelete] = useState<AiConversation | null>(null)
  const logRef = useRef<HTMLDivElement>(null)
  const model = modelDraft ?? settings?.model ?? ''
  const baseUrl = baseUrlDraft ?? settings?.baseUrl ?? ''
  const usesFreeBackend = baseUrl.trim().length > 0

  // Follow the conversation as it grows.
  useEffect(() => {
    const log = logRef.current
    if (log) log.scrollTop = log.scrollHeight
  }, [messages, busy, streaming, pendingUser])

  const needsKey = settings !== null && !usesFreeBackend && !settings.apiKeySet
  const canSend = !busy && draft.trim().length > 0

  function send(text: string) {
    const trimmed = text.trim()
    if (!trimmed || busy) return
    onSend(trimmed)
    setDraft('')
  }

  function saveSettings() {
    // An empty key field means "leave the stored key alone".
    onSaveSettings({
      apiKey: apiKey.trim() ? apiKey.trim() : null,
      baseUrl: baseUrl.trim(),
      model: model.trim(),
    })
    setApiKey('')
    setModelDraft(null)
    setBaseUrlDraft(null)
    setShowSettings(false)
  }

  function clearKey() {
    onSaveSettings({ apiKey: '' })
    setApiKey('')
  }

  return (
    <aside className="assistant" aria-label="JARVIS assistant">
      <header>
        <div className="orb" aria-hidden />
        <div>
          <b>JARVIS</b>
          <span>
            {settings?.model
              ? `${usesFreeBackend ? 'Custom' : 'Claude'} · ${settings.model}`
              : 'Assistant'}
          </span>
        </div>
        <Button
          type="button"
          variant="ghost"
          size="icon"
          aria-label="Assistant settings"
          aria-expanded={showSettings}
          onClick={() => setShowSettings((open) => !open)}
          style={{ marginLeft: 'auto' }}
        >
          <Settings aria-hidden />
        </Button>
      </header>

      {showSettings && (
        <div className="assistant-settings" role="group" aria-label="Assistant settings">
          <label>
            <span>Base URL (free / OpenAI-compatible — leave empty for Anthropic)</span>
            <Input
              value={baseUrl}
              onChange={(e) => setBaseUrlDraft(e.target.value)}
              placeholder="http://localhost:11434/v1"
              aria-label="Base URL"
            />
          </label>
          <label>
            <span>API key {usesFreeBackend ? '(optional)' : ''}</span>
            <Input
              type="password"
              value={apiKey}
              onChange={(e) => setApiKey(e.target.value)}
              placeholder={settings?.apiKeySet ? 'Key saved — type to replace' : 'sk-ant-…'}
              aria-label="API key"
            />
          </label>
          <label>
            <span>Model</span>
            <Input
              value={model}
              onChange={(e) => setModelDraft(e.target.value)}
              placeholder={usesFreeBackend ? 'openai/gpt-oss-20b' : 'claude-sonnet-5-5'}
              aria-label="Model id"
            />
          </label>
          <div className="row">
            <Button type="button" size="sm" onClick={saveSettings}>
              Save
            </Button>
            {settings?.apiKeySet && (
              <Button type="button" size="sm" variant="outline" onClick={clearKey}>
                Remove key
              </Button>
            )}
          </div>
          <p>
            Leave the key empty to use the hosted proxy. Or set a Base URL to any
            OpenAI-compatible server (Groq, Ollama) and supply your own key.
            Settings stay on this device.
          </p>
        </div>
      )}

      {conversations.length > 0 && (
        <div className="assistant-threads">
          <Select
            value={activeConversationId === null ? undefined : String(activeConversationId)}
            onValueChange={(value) => onSelectConversation(Number(value))}
          >
            <SelectTrigger size="sm" aria-label="Conversation" style={{ flex: 1, minWidth: 0 }}>
              <SelectValue placeholder="New chat" />
            </SelectTrigger>
            <SelectContent>
              {conversations.map((conversation) => (
                <SelectItem key={conversation.id} value={String(conversation.id)}>
                  {conversation.title ?? `Chat ${conversation.id}`}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
          <Button
            type="button"
            variant="outline"
            size="icon"
            aria-label="New chat"
            onClick={onNewConversation}
          >
            <Plus aria-hidden />
          </Button>
          {activeConversationId !== null && (
            <Button
              type="button"
              variant="outline"
              size="icon"
              aria-label="Delete chat"
              onClick={() =>
                setPendingDelete(
                  conversations.find((c) => c.id === activeConversationId) ?? null,
                )
              }
            >
              <Trash2 aria-hidden />
            </Button>
          )}
        </div>
      )}

      <div className="log" aria-live="polite" ref={logRef}>
        {messages.length === 0 && !busy && !streaming && !pendingUser ? (
          <p style={{ color: 'var(--color-muted)' }}>
            {needsKey
              ? 'Add an Anthropic API key, or set a free base URL, to switch me on.'
              : 'Hello. Ask me to plan your day, check your progress or add a task.'}
          </p>
        ) : (
          messages.map((message) => (
            <p key={message.id} className={`msg ${message.role}`}>
              {message.content}
            </p>
          ))
        )}
        {pendingUser !== null && <p className="msg user">{pendingUser}</p>}
        {streaming ? (
          <p className="msg assistant streaming">{streaming}</p>
        ) : (
          busy && <p className="msg assistant thinking">JARVIS is thinking…</p>
        )}
        {needsKey && (
          <Button type="button" variant="outline" size="sm" onClick={() => setShowSettings(true)}>
            Configure assistant
          </Button>
        )}
      </div>

      <footer>
        <div className="chips" style={{ display: 'flex', gap: '0.375rem', flexWrap: 'wrap' }}>
          {SUGGESTIONS.map((suggestion) => (
            <Button
              key={suggestion}
              variant="outline"
              size="sm"
              disabled={busy}
              onClick={() => send(suggestion)}
            >
              {suggestion}
            </Button>
          ))}
        </div>
        <form
          onSubmit={(e) => {
            e.preventDefault()
            send(draft)
          }}
        >
          <input
            value={draft}
            onChange={(e) => setDraft(e.target.value)}
            placeholder="Ask JARVIS…"
            aria-label="Message JARVIS"
            disabled={busy}
          />
          <Button type="button" variant="outline" size="icon" aria-label="Voice input (coming soon)" disabled>
            <Mic aria-hidden />
          </Button>
          <Button type="submit" disabled={!canSend}>
            Send
          </Button>
        </form>
      </footer>

      {pendingDelete && (
        <ConfirmDialog
          title="Delete this chat?"
          body="The conversation and its messages will be permanently deleted."
          onCancel={() => setPendingDelete(null)}
          onConfirm={() => {
            onDeleteConversation(pendingDelete.id)
            setPendingDelete(null)
          }}
        />
      )}
    </aside>
  )
}
