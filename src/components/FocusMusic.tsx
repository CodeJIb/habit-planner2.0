import { FolderOpen, Music, Pause, Play, Volume2, VolumeX } from 'lucide-react'
import { useEffect, useRef, useState } from 'react'

/**
 * Built-in focus track: 40Hz gamma binaural beats, synthesised with the Web
 * Audio API. Two detuned sine waves are panned hard left/right so the brain
 * perceives a 40Hz beat. This needs no network and no YouTube embed, so it
 * works offline and on mobile where the YouTube iframe is restricted.
 */
const CARRIER_HZ = 200
const BEAT_HZ = 40
const LEVEL = 0.18

/** IndexedDB home for the audio the user picks, so it survives a restart. */
const DB_NAME = 'habit-planner'
const STORE = 'focus-audio'
const RECORD_KEY = 'custom'
/** Remembers whether the built-in track or the device track was last chosen. */
const SOURCE_KEY = 'focusMusicSource'

interface SavedAudio {
  blob: Blob
  name: string
}

function openDb(): Promise<IDBDatabase> {
  return new Promise((resolve, reject) => {
    const request = indexedDB.open(DB_NAME, 1)
    request.onupgradeneeded = () => {
      if (!request.result.objectStoreNames.contains(STORE)) {
        request.result.createObjectStore(STORE)
      }
    }
    request.onsuccess = () => resolve(request.result)
    request.onerror = () => reject(request.error)
  })
}

async function saveAudio(audio: SavedAudio): Promise<void> {
  const db = await openDb()
  await new Promise<void>((resolve, reject) => {
    const tx = db.transaction(STORE, 'readwrite')
    tx.objectStore(STORE).put(audio, RECORD_KEY)
    tx.oncomplete = () => resolve()
    tx.onerror = () => reject(tx.error)
  })
  db.close()
}

async function loadAudio(): Promise<SavedAudio | null> {
  const db = await openDb()
  const saved = await new Promise<SavedAudio | null>((resolve, reject) => {
    const tx = db.transaction(STORE, 'readonly')
    const request = tx.objectStore(STORE).get(RECORD_KEY)
    request.onsuccess = () => resolve((request.result as SavedAudio | undefined) ?? null)
    request.onerror = () => reject(request.error)
  })
  db.close()
  return saved
}

/** The live binaural graph, created on first play. */
interface Binaural {
  ctx: AudioContext
  gain: GainNode
  oscillators: OscillatorNode[]
}

type AudioContextCtor = typeof AudioContext

function audioContextCtor(): AudioContextCtor | null {
  if (typeof window === 'undefined') return null
  const w = window as unknown as { AudioContext?: AudioContextCtor; webkitAudioContext?: AudioContextCtor }
  return w.AudioContext ?? w.webkitAudioContext ?? null
}

function formatTime(seconds: number): string {
  if (!Number.isFinite(seconds) || seconds < 0) return '0:00'
  const mm = Math.floor(seconds / 60)
  const ss = String(Math.floor(seconds % 60)).padStart(2, '0')
  return `${mm}:${ss}`
}

function stripExtension(name: string): string {
  return name.replace(/\.[^.]+$/, '')
}

interface LocalTrack {
  name: string
  url: string
}

interface Props {
  /** True while the focus timer is actively counting down. */
  playing: boolean
  /** Start or pause the focus session; the music stays in step with it. */
  onTogglePlay: () => void
  /** True while a session is starting, so the button cannot be double-clicked. */
  busy?: boolean
}

/**
 * Spotify-style focus music bar. The green button starts/pauses the focus
 * session and the audio follows it. The user can play the built-in binaural
 * beats (synthesised locally) or pick their own track from the device (kept in
 * IndexedDB, so the choice survives a restart).
 */
export function FocusMusic({ playing, onTogglePlay, busy = false }: Props) {
  const audioRef = useRef<HTMLAudioElement>(null)
  const fileRef = useRef<HTMLInputElement>(null)
  const urlRef = useRef<string | null>(null)
  const binauralRef = useRef<Binaural | null>(null)
  const mutedRef = useRef(false)
  const [local, setLocal] = useState<LocalTrack | null>(null)
  const [useLocal, setUseLocal] = useState(false)
  const [muted, setMuted] = useState(false)
  const [elapsed, setElapsed] = useState(0)
  const [progress, setProgress] = useState({ current: 0, duration: 0 })

  // Keep the latest mute flag available to the (unrelated) graph-creation effect.
  useEffect(() => {
    mutedRef.current = muted
  }, [muted])

  // Tear the synthesised track down when the bar unmounts.
  useEffect(() => {
    return () => {
      const bin = binauralRef.current
      binauralRef.current = null
      if (bin) void bin.ctx.close().catch(() => {})
    }
  }, [])

  // Restore any track the user picked previously.
  useEffect(() => {
    let cancelled = false
    void loadAudio()
      .then((saved) => {
        if (cancelled || !saved) return
        const url = URL.createObjectURL(saved.blob)
        urlRef.current = url
        setLocal({ name: saved.name, url })
        setUseLocal(localStorage.getItem(SOURCE_KEY) !== 'builtin')
      })
      .catch(() => {
        // No stored track, or storage unavailable: keep the built-in track.
      })
    return () => {
      cancelled = true
      if (urlRef.current) URL.revokeObjectURL(urlRef.current)
    }
  }, [])

  // Built-in track: start/resume or suspend the binaural graph with the timer.
  useEffect(() => {
    if (useLocal) return
    const Ctor = audioContextCtor()
    if (!Ctor) return
    if (playing) {
      let bin = binauralRef.current
      if (!bin) {
        const ctx = new Ctor()
        const gain = ctx.createGain()
        gain.gain.value = mutedRef.current ? 0 : LEVEL
        gain.connect(ctx.destination)
        const oscillators: OscillatorNode[] = []
        for (const [freq, pan] of [
          [CARRIER_HZ, -1],
          [CARRIER_HZ + BEAT_HZ, 1],
        ] as const) {
          const osc = ctx.createOscillator()
          osc.type = 'sine'
          osc.frequency.value = freq
          const panner = ctx.createStereoPanner()
          panner.pan.value = pan
          osc.connect(panner).connect(gain)
          osc.start()
          oscillators.push(osc)
        }
        bin = { ctx, gain, oscillators }
        binauralRef.current = bin
      }
      void bin.ctx.resume().catch(() => {})
    } else {
      const bin = binauralRef.current
      if (bin) void bin.ctx.suspend().catch(() => {})
    }
  }, [playing, useLocal])

  // Local track follows the timer too.
  useEffect(() => {
    const audio = audioRef.current
    if (!audio) return
    if (useLocal && playing) void audio.play().catch(() => {})
    else audio.pause()
  }, [playing, useLocal, local])

  // Mute whichever source is active.
  useEffect(() => {
    const bin = binauralRef.current
    if (bin) bin.gain.gain.setTargetAtTime(muted ? 0 : LEVEL, bin.ctx.currentTime, 0.05)
    if (audioRef.current) audioRef.current.muted = muted
  }, [muted])

  // Advance the clock: elapsed for the endless built-in track, position for local.
  useEffect(() => {
    if (!playing) return
    const id = window.setInterval(() => {
      if (useLocal) {
        const audio = audioRef.current
        if (audio) setProgress({ current: audio.currentTime, duration: audio.duration })
      } else {
        setElapsed((s) => s + 1)
      }
    }, 1000)
    return () => window.clearInterval(id)
  }, [playing, useLocal])

  async function pick(file: File) {
    try {
      await saveAudio({ blob: file, name: file.name })
    } catch {
      // Storage full or blocked: still play it for this session.
    }
    if (urlRef.current) URL.revokeObjectURL(urlRef.current)
    const url = URL.createObjectURL(file)
    urlRef.current = url
    setLocal({ name: file.name, url })
    setUseLocal(true)
    localStorage.setItem(SOURCE_KEY, 'device')
  }

  function useBuiltIn() {
    setUseLocal(false)
    setElapsed(0)
    localStorage.setItem(SOURCE_KEY, 'builtin')
  }

  const showLocal = useLocal && local !== null
  const pct =
    showLocal && progress.duration > 0
      ? Math.min(100, (progress.current / progress.duration) * 100)
      : 0
  const title = showLocal ? stripExtension(local.name) : '40Hz Gamma Binaural Beats'
  const artist = showLocal ? 'From your device' : 'Synthesised · loops for your session'
  const current = showLocal ? progress.current : elapsed
  const endLabel = showLocal ? formatTime(progress.duration) : '∞'

  return (
    <div className="focus-music">
      <audio ref={audioRef} src={local?.url} loop />
      <input
        ref={fileRef}
        type="file"
        accept="audio/*"
        hidden
        onChange={(e) => {
          const file = e.target.files?.[0]
          if (file) void pick(file)
          e.target.value = ''
        }}
      />

      <div
        className={`focus-music-art focus-music-art-icon${showLocal ? '' : ' focus-music-art-live'}`}
        aria-hidden
      >
        <Music className="h-6 w-6" />
      </div>

      <div className="focus-music-body">
        <span className="focus-music-title" title={showLocal ? local.name : undefined}>
          {title}
        </span>
        <span className="focus-music-artist">{artist}</span>
        <div className="focus-music-progress">
          <span>{formatTime(current)}</span>
          <div className="focus-music-track">
            <div
              className={`focus-music-fill${showLocal ? '' : ' focus-music-fill--loop'}`}
              style={showLocal ? { width: `${pct}%` } : undefined}
            />
          </div>
          <span>{endLabel}</span>
        </div>
      </div>

      <button
        type="button"
        className="focus-music-icon"
        aria-label="Use the built-in binaural beats"
        aria-pressed={!showLocal}
        title="Built-in binaural beats"
        onClick={useBuiltIn}
      >
        <Music className="h-5 w-5" aria-hidden />
      </button>
      <button
        type="button"
        className="focus-music-icon"
        aria-label="Add music or a sound from your device"
        title="Add from your device"
        onClick={() => fileRef.current?.click()}
      >
        <FolderOpen className="h-5 w-5" aria-hidden />
      </button>
      <button
        type="button"
        className="focus-music-icon"
        aria-label={muted ? 'Unmute focus music' : 'Mute focus music'}
        aria-pressed={muted}
        onClick={() => setMuted((m) => !m)}
      >
        {muted ? <VolumeX className="h-5 w-5" aria-hidden /> : <Volume2 className="h-5 w-5" aria-hidden />}
      </button>
      <button
        type="button"
        className="focus-music-play"
        aria-label={playing ? 'Pause focus session' : 'Start focus session'}
        disabled={busy}
        onClick={onTogglePlay}
      >
        {playing ? <Pause className="h-5 w-5" aria-hidden /> : <Play className="h-5 w-5" aria-hidden />}
      </button>
    </div>
  )
}
