import { Mic, Plus, Settings, Trash2, Volume2, VolumeX } from 'lucide-react'
import { useEffect, useRef, useState } from 'react'
import { Button } from './ui/button'
import { Input } from './ui/input'
import { ConfirmDialog } from './ConfirmDialog'
import { JarvisHud } from './JarvisHud'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from './ui/select'
import type {
  AiConversation,
  AiMessage,
  AiSettings,
  AiSettingsInput,
  CloudVoice,
  TtsSettings,
  TtsSettingsInput,
} from '../types'

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
  /** Text-to-speech settings, or null before they have loaded. */
  tts: TtsSettings | null
  /** False where the platform can speak at all (no Web Speech API, no Tauri). */
  ttsSupported: boolean
  /** System voices the OS offers, for the "system" dropdown. */
  ttsVoices: SpeechSynthesisVoice[]
  /** Edge neural voices for the "cloud" dropdown. */
  cloudVoices: CloudVoice[]
  onToggleTts: () => void
  onSaveTts: (input: TtsSettingsInput) => void
  onTestVoice: () => void
  /** Opens the OS speech settings so a new system voice can be installed. */
  onInstallVoice: () => void
}

const SUGGESTIONS = ['Plan my day', 'I need to vent', 'How much XP to level up?']

/** Radix Select forbids an empty item value, so "auto" needs a sentinel. */
const AUTO_VOICE = '__auto__'
/** Sentinel for "let JARVIS pick the best installed system voice". */
const AUTO_CLOUD = '__auto_cloud__'

/** Sort British voices first — that is the JARVIS sound — then by locale. */
function sortCloudVoices(voices: CloudVoice[]): CloudVoice[] {
  return [...voices].sort((a, b) => {
    const aGb = a.locale.toLowerCase().startsWith('en-gb') ? 0 : 1
    const bGb = b.locale.toLowerCase().startsWith('en-gb') ? 0 : 1
    if (aGb !== bGb) return aGb - bGb
    return a.label.localeCompare(b.label)
  })
}

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
  tts,
  ttsSupported,
  ttsVoices,
  cloudVoices,
  onToggleTts,
  onSaveTts,
  onTestVoice,
  onInstallVoice,
}: Props) {
  const [draft, setDraft] = useState('')
  const [showSettings, setShowSettings] = useState(false)
  const [apiKey, setApiKey] = useState('')
  /** Local overrides while editing; each falls back to the saved value. */
  const [modelDraft, setModelDraft] = useState<string | null>(null)
  const [baseUrlDraft, setBaseUrlDraft] = useState<string | null>(null)
  const [rateDraft, setRateDraft] = useState<number | null>(null)
  const [pitchDraft, setPitchDraft] = useState<number | null>(null)
  const [pendingDelete, setPendingDelete] = useState<AiConversation | null>(null)
  const logRef = useRef<HTMLDivElement>(null)
  const model = modelDraft ?? settings?.model ?? ''
  const baseUrl = baseUrlDraft ?? settings?.baseUrl ?? ''
  const usesFreeBackend = baseUrl.trim().length > 0
  const ttsEnabled = tts?.enabled ?? false
  const ttsSource = tts?.source ?? 'cloud'
  const isCloud = ttsSource === 'cloud'
  // JARVIS's real voice is British. The cloud catalogue always offers it
  // (Ryan/Thomas Neural); the system list usually does not, so the "install a
  // voice" hint only appears for the offline system source.
  const hasBritishVoice = isCloud
    ? true
    : ttsVoices.some((voice) =>
        voice.lang.toLowerCase().replace('_', '-').startsWith('en-gb'),
      )
  const sortedCloudVoices = sortCloudVoices(cloudVoices)
  const rate = rateDraft ?? tts?.rate ?? 1
  const pitch = pitchDraft ?? tts?.pitch ?? 1

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

  // Sliders save on release (pointer or keyboard), not on every pixel of drag.
  function commitRate(value: number) {
    setRateDraft(null)
    onSaveTts({ rate: value })
  }

  function commitPitch(value: number) {
    setPitchDraft(null)
    onSaveTts({ pitch: value })
  }

  return (
    <aside className="assistant" aria-label="JARVIS assistant">
      <header>
        <div className="hud-status">
          <span>SYS // ONLINE</span>
          <span>STARK INDUSTRIES</span>
        </div>
        <Button
          type="button"
          variant="ghost"
          size="icon"
          aria-label="Assistant settings"
          aria-expanded={showSettings}
          onClick={() => setShowSettings((open) => !open)}
        >
          <Settings aria-hidden />
        </Button>
      </header>

      <div className="hud-title">J.A.R.V.I.S.</div>
      <JarvisHud talking={busy} />

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

          {ttsSupported && (
            <fieldset className="voice-settings">
              <legend>Voice</legend>
              <label>
                <span>Sound source</span>
                <Select value={ttsSource} onValueChange={(value) => onSaveTts({ source: value as 'cloud' | 'system' })}>
                  <SelectTrigger size="sm" style={{ width: '100%' }} aria-label="Sound source">
                    <SelectValue placeholder="Cloud (JARVIS)" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="cloud">Cloud — neural voices (online)</SelectItem>
                    <SelectItem value="system">System — installed voices (offline)</SelectItem>
                  </SelectContent>
                </Select>
              </label>

              {isCloud ? (
                <label>
                  <span>Voice</span>
                  <Select
                    value={tts?.cloudVoice ? tts.cloudVoice : AUTO_CLOUD}
                    onValueChange={(value) =>
                      onSaveTts({ cloudVoice: value === AUTO_CLOUD ? 'en-GB-RyanNeural' : value })
                    }
                  >
                    <SelectTrigger size="sm" style={{ width: '100%' }} aria-label="Cloud voice">
                      <SelectValue placeholder="Ryan (British)" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value={AUTO_CLOUD}>Ryan — British male (recommended)</SelectItem>
                      {sortedCloudVoices.map((voice) => (
                        <SelectItem key={voice.id} value={voice.id}>
                          {voice.label}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </label>
              ) : (
                <label>
                  <span>Voice</span>
                  <Select
                    value={tts?.voice ? tts.voice : AUTO_VOICE}
                    onValueChange={(value) =>
                      onSaveTts({ voice: value === AUTO_VOICE ? '' : value })
                    }
                  >
                    <SelectTrigger size="sm" style={{ width: '100%' }} aria-label="Voice">
                      <SelectValue placeholder="Auto (butler voice)" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value={AUTO_VOICE}>Auto (butler voice)</SelectItem>
                      {ttsVoices.map((voice) => (
                        <SelectItem key={voice.name} value={voice.name}>
                          {voice.name} ({voice.lang})
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </label>
              )}

              {!hasBritishVoice && (
                <p>
                  I have no British voice yet — that is what JARVIS sounds like. Install{' '}
                  <b>Daniel (United Kingdom)</b> in Windows, keep this on{' '}
                  <b>Auto (butler voice)</b>, and restart the app.
                  <br />
                  <Button
                    type="button"
                    size="sm"
                    variant="outline"
                    onClick={onInstallVoice}
                    style={{ marginTop: '0.5rem' }}
                  >
                    Open voice settings
                  </Button>
                </p>
              )}

              <label>
                <span>Speed — {rate.toFixed(2)}×</span>
                <input
                  type="range"
                  min={0.5}
                  max={2}
                  step={0.05}
                  value={rate}
                  aria-label="Speech speed"
                  onChange={(e) => setRateDraft(Number(e.target.value))}
                  onPointerUp={(e) => commitRate(Number(e.currentTarget.value))}
                  onKeyUp={(e) => commitRate(Number(e.currentTarget.value))}
                />
              </label>

              {!isCloud && (
                <label>
                  <span>Pitch — {pitch.toFixed(2)}</span>
                  <input
                    type="range"
                    min={0.5}
                    max={2}
                    step={0.05}
                    value={pitch}
                    aria-label="Speech pitch"
                    onChange={(e) => setPitchDraft(Number(e.target.value))}
                    onPointerUp={(e) => commitPitch(Number(e.currentTarget.value))}
                    onKeyUp={(e) => commitPitch(Number(e.currentTarget.value))}
                  />
                </label>
              )}

              <div className="row">
                <Button type="button" size="sm" variant="outline" onClick={onTestVoice}>
                  Test voice
                </Button>
              </div>
            </fieldset>
          )}

          <p>
            Paste an Anthropic API key to chat directly from this device. Or set a
            Base URL to any OpenAI-compatible server (Groq, Ollama, Gemini) instead.
            Settings never leave your machine.
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
              : 'At your service. Tell me how the day is going, or ask me to plan it — I am listening.'}
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
          {ttsSupported && (
            <Button
              type="button"
              variant="outline"
              size="icon"
              aria-label={ttsEnabled ? 'Turn voice off' : 'Turn voice on'}
              aria-pressed={ttsEnabled}
              onClick={onToggleTts}
            >
              {ttsEnabled ? <Volume2 aria-hidden /> : <VolumeX aria-hidden />}
            </Button>
          )}
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
