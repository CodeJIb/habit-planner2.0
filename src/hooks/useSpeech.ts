import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import { isTauri, synthSpeech } from '../lib/api'

/** Settings the hook reads when it builds each utterance. */
export interface SpeechOptions {
  /** Where speech is produced: "cloud" (Edge neural) or "system" (offline). */
  source: 'cloud' | 'system'
  /** Preferred system voice name from the OS. Empty means "auto-pick". */
  voice: string
  /** Edge voice short name when source is "cloud". */
  cloudVoice: string
  /** 1.0 is normal speed. */
  rate: number
  /** 1.0 is normal pitch. System voice only. */
  pitch: number
}

export interface Speech {
  /** False where the Web Speech API is missing (the UI then hides the button). */
  supported: boolean
  /** True from the first queued line until the last one finishes. */
  speaking: boolean
  /** True while a cloud line is being synthesised (network in flight). */
  synthBusy: boolean
  /** Voices the OS offers; may be empty until the `voiceschanged` event fires. */
  voices: SpeechSynthesisVoice[]
  /** Speak one line now, replacing anything already queued. */
  speak: (text: string) => void
  /** Add one line to the queue. */
  enqueue: (text: string) => void
  /** Stop immediately and drop the queue. */
  cancel: () => void
  /** Unlock speech on the first user gesture (mobile webviews require this). */
  prime: () => void
}

/**
 * Score a voice for a calm, British butler delivery — the JARVIS brief. Voices
 * differ per OS, so nothing is hardcoded: British English wins outright, then
 * any male-sounding English voice, and female-sounding names are pushed down so
 * "Auto" lands on the most JARVIS-like voice actually installed.
 */
function scoreVoice(voice: SpeechSynthesisVoice): number {
  const name = voice.name.toLowerCase()
  const lang = voice.lang.toLowerCase().replace('_', '-')
  let points = 0
  if (lang.startsWith('en-gb') || name.includes('uk') || name.includes('british')) points += 6
  else if (lang.startsWith('en')) points += 1
  if (/\bmale\b/.test(name)) points += 3
  if (/\b(daniel|george|arthur|ryan|guy|oliver|thomas|alfie|alex)\b/.test(name)) points += 3
  if (
    name.includes('female') ||
    /(zira|samantha|susan|karen|moira|sonia|libby|hazel|catherine|victoria)/.test(name)
  )
    points -= 5
  return points
}

/** Pick the saved voice by name, or the best-scoring one when none is saved. */
function selectVoice(
  voices: SpeechSynthesisVoice[],
  preferred: string,
): SpeechSynthesisVoice | null {
  if (voices.length === 0) return null
  const wanted = preferred.trim().toLowerCase()
  if (wanted) {
    const exact = voices.find((voice) => voice.name.toLowerCase() === wanted)
    if (exact) return exact
  }
  return [...voices].sort((a, b) => scoreVoice(b) - scoreVoice(a))[0]
}

/**
 * Thin wrapper over `window.speechSynthesis`. Every entry point is a no-op when
 * the platform lacks the API, so the app keeps working even with no speech.
 *
 * Note: this hook always speaks when asked. The master on/off switch lives with
 * the caller, which decides whether to enqueue streamed replies at all — that
 * way the "Test voice" button still works while the toggle is off.
 */
export function useSpeech(options: SpeechOptions): Speech {
  const supported = useMemo(() => {
    if (typeof window === 'undefined') return false
    return 'speechSynthesis' in window || isTauri
  }, [])
  const [voices, setVoices] = useState<SpeechSynthesisVoice[]>([])
  const [speaking, setSpeaking] = useState(false)
  /** True while a cloud synthesis request is in flight. */
  const [synthBusy, setSynthBusy] = useState(false)

  // Latest options, so a queued utterance always uses the current voice shape.
  const optionsRef = useRef(options)
  useEffect(() => {
    optionsRef.current = options
  }, [options])

  // Bumped on every cancel so late onend/onerror callbacks are ignored.
  const generation = useRef(0)
  const queued = useRef(0)

  // Cloud audio: MP3s are synthesised in Rust, played one after another from
  // an object-URL queue. `audioRef` holds the element so `cancel` can stop it.
  const audioRef = useRef<HTMLAudioElement | null>(null)
  const cloudQueue = useRef<string[]>([])
  const cloudBusy = useRef(false)
  // Lets the clip-finished handler kick off the next clip without the callback
  // having to reference itself while it is still being initialised.
  const pumpRef = useRef<() => void>(() => {})

  /** Play the next synthesised clip, if the queue is not empty. */
  const pumpCloud = useCallback(() => {
    if (cloudBusy.current) return
    const url = cloudQueue.current.shift()
    if (!url) return
    cloudBusy.current = true
    setSynthBusy(true)

    const audio = audioRef.current ?? new Audio()
    audioRef.current = audio
    const gen = generation.current
    audio.src = url
    const finish = () => {
      URL.revokeObjectURL(url)
      audio.onended = null
      audio.onerror = null
      audio.src = ''
      // Ignore a late callback from a queue that was cancelled meanwhile.
      if (gen !== generation.current) return
      cloudBusy.current = false
      setSynthBusy(false)
      if (cloudQueue.current.length === 0) setSpeaking(false)
      else pumpRef.current()
    }
    audio.onended = finish
    audio.onerror = finish
    void audio.play().catch(finish)
  }, [])
  useEffect(() => {
    pumpRef.current = pumpCloud
  }, [pumpCloud])

  // Load voices. getVoices() is often empty on the first call, so we also
  // listen for `voiceschanged` (fires once the OS finishes enumerating them).
  useEffect(() => {
    if (!('speechSynthesis' in window)) return
    const synth = window.speechSynthesis
    const load = () => setVoices(synth.getVoices())
    load()
    synth.addEventListener('voiceschanged', load)
    return () => synth.removeEventListener('voiceschanged', load)
  }, [])

  const cancel = useCallback(() => {
    generation.current += 1
    queued.current = 0

    // Drop any cloud clips still waiting, and stop the one playing now.
    for (const url of cloudQueue.current.splice(0)) URL.revokeObjectURL(url)
    cloudBusy.current = false
    setSynthBusy(false)
    if (audioRef.current) {
      audioRef.current.pause()
      audioRef.current.src = ''
    }

    if ('speechSynthesis' in window) window.speechSynthesis.cancel()
    setSpeaking(false)
  }, [])

  // Enqueue one line: synthesise on the cloud path, or speak via the OS.
  const enqueueSystem = useCallback((text: string) => {
    const synth = window.speechSynthesis
    const utterance = new SpeechSynthesisUtterance(text)
    const chosen = selectVoice(synth.getVoices(), optionsRef.current.voice)
    if (chosen) utterance.voice = chosen
    utterance.rate = optionsRef.current.rate
    utterance.pitch = optionsRef.current.pitch

    const gen = generation.current
    queued.current += 1
    setSpeaking(true)

    const done = () => {
      // Ignore callbacks from a queue that was cancelled in the meantime.
      if (gen !== generation.current) return
      queued.current = Math.max(0, queued.current - 1)
      if (queued.current === 0) setSpeaking(false)
    }
    utterance.onend = done
    utterance.onerror = done
    synth.speak(utterance)
  }, [])

  const enqueue = useCallback(
    (text: string) => {
      const trimmed = text.trim()
      if (!trimmed) return
      const { source, cloudVoice, rate } = optionsRef.current

      // Cloud needs Tauri (Rust does the fetching). Any failure — offline, bad
      // voice, service down — quietly falls back to the system voice so JARVIS
      // keeps talking.
      if (source === 'cloud' && isTauri) {
        const gen = generation.current
        setSpeaking(true)
        void synthSpeech(trimmed, cloudVoice, rate)
          .then((url) => {
            if (gen !== generation.current) {
              URL.revokeObjectURL(url)
              return
            }
            cloudQueue.current.push(url)
            pumpCloud()
          })
          .catch(() => {
            if (gen !== generation.current) return
            if (cloudQueue.current.length === 0 && !cloudBusy.current) setSpeaking(false)
            enqueueSystem(trimmed)
          })
        return
      }

      if (!('speechSynthesis' in window)) return
      enqueueSystem(trimmed)
    },
    [enqueueSystem, pumpCloud],
  )

  const speak = useCallback(
    (text: string) => {
      cancel()
      enqueue(text)
    },
    [cancel, enqueue],
  )

  const prime = useCallback(() => {
    // The cloud voice plays through an <audio> element, which needs no OS
    // unlock; only the system voice needs the Android/iOS gesture dance.
    if (!('speechSynthesis' in window)) return
    const synth = window.speechSynthesis
    // `resume` plus a silent utterance counts as the first user gesture that
    // Android and iOS webviews require before they let us speak.
    synth.resume()
    const cue = new SpeechSynthesisUtterance(' ')
    cue.volume = 0
    synth.speak(cue)
  }, [])

  // Stop everything when the component that owns the hook unmounts.
  useEffect(
    () => () => {
      generation.current += 1
      for (const url of cloudQueue.current.splice(0)) URL.revokeObjectURL(url)
      if (audioRef.current) {
        audioRef.current.pause()
        audioRef.current.src = ''
      }
      if ('speechSynthesis' in window) window.speechSynthesis.cancel()
    },
    [],
  )

  // Chromium (which includes the WebView2 the desktop app runs in, and the
  // Android WebView) silently stops a single utterance after roughly 15
  // seconds. Periodically pausing and resuming the queue resets that internal
  // timer so long replies — a whole spoken paragraph — finish out loud.
  // The cloud <audio> path has no such limit, so it is left alone.
  useEffect(() => {
    if (!('speechSynthesis' in window) || !speaking) return
    const synth = window.speechSynthesis
    const id = window.setInterval(() => {
      if (!synth.speaking || synth.paused) return
      synth.pause()
      synth.resume()
    }, 10_000)
    return () => window.clearInterval(id)
  }, [speaking])

  return { supported, speaking, synthBusy, voices, speak, enqueue, cancel, prime }
}
