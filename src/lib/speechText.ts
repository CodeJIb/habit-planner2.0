// Helpers that turn streamed markdown into clean, spoken sentences.
//
// The assistant streams raw markdown token by token. We buffer the text and pull
// out one sentence at a time, so JARVIS starts speaking before the whole reply
// has arrived — and so no single utterance is long enough to make a webview
// silently drop it.

export interface DrainedSpeech {
  /** Complete sentences ready to speak, in order. */
  sentences: string[]
  /** The unfinished tail to keep buffering. */
  rest: string
  /** Whether we are currently inside a fenced code block. */
  inCode: boolean
}

const FENCE = '```'
const SENTENCE_END = /[.!?](\s|$)/g

/**
 * Walk the buffer, drop any fenced code, and return whole lines plus every
 * finished sentence found in the unfinished last line.
 *
 * `startInCode` carries the fence state across calls, because a ``` fence can
 * arrive split across several streamed deltas.
 */
export function drainSpeech(buffer: string, startInCode: boolean): DrainedSpeech {
  const sentences: string[] = []
  let inCode = startInCode

  const lines = buffer.split('\n')
  const partial = lines.pop() ?? ''

  for (const raw of lines) {
    const line = raw.trim()
    if (line.startsWith(FENCE)) {
      inCode = !inCode
      continue
    }
    if (inCode || !line) continue
    sentences.push(line)
  }

  const partialTrimmed = partial.trimStart()
  if (partialTrimmed.startsWith(FENCE)) {
    // A fence marker is still streaming in; drop it and remember the new state.
    inCode = !inCode
    return { sentences, rest: '', inCode }
  }
  if (inCode) {
    // Inside a code block: keep the tail so a closing fence can still be seen,
    // but never emit it (code is not read aloud).
    return { sentences, rest: partial, inCode }
  }

  // Pull complete sentences out of the unfinished line.
  let rest = partial
  let lastEnd = 0
  SENTENCE_END.lastIndex = 0
  let match: RegExpExecArray | null
  while ((match = SENTENCE_END.exec(partial)) !== null) {
    const end = match.index + 1
    const chunk = partial.slice(lastEnd, end).trim()
    if (chunk) sentences.push(chunk)
    lastEnd = end
    rest = partial.slice(lastEnd)
  }

  return { sentences, rest, inCode }
}

/**
 * Strip markdown so the spoken text reads naturally: links become their labels,
 * code spans lose their backticks, and emphasis/heading/list marks and emoji go.
 */
export function cleanForSpeech(text: string): string {
  return text
    .replace(/!\[[^\]]*\]\([^)]*\)/g, '') // images: drop entirely
    .replace(/\[([^\]]+)\]\([^)]*\)/g, '$1') // links: keep the label
    .replace(/`([^`]+)`/g, '$1') // inline code: keep the text
    .replace(/[*_>#]+/g, '') // emphasis, headings, blockquotes
    .replace(/^\s*(?:[-+*]|\d+\.)\s+/gm, '') // list markers
    .replace(/[\p{Extended_Pictographic}\u{FE0F}]/gu, '') // emoji + variation selectors
    .replace(/\s+/g, ' ')
    .trim()
}
