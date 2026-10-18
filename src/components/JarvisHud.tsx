interface Props {
  /** True while JARVIS is thinking or speaking; animates the core and bars. */
  talking?: boolean
}

/**
 * Full-size arc-reactor showpiece for the JARVIS HUD: the multi-ring reactor
 * with a pulsing core and a five-bar voice waveform, drawn as one SVG with CSS
 * animations (no JavaScript while it animates). Used at the top of the
 * assistant panel and the Today hero.
 */
export function JarvisHud({ talking = false }: Props) {
  return (
    <svg
      className="reactor"
      viewBox="0 0 200 200"
      data-talking={talking}
      aria-hidden
    >
      <defs>
        <radialGradient id="reactor-core" cx="50%" cy="50%" r="50%">
          <stop offset="0%" stopColor="#ffffff" />
          <stop offset="35%" stopColor="#9bf0ff" />
          <stop offset="100%" stopColor="#0b4a6e" stopOpacity="0.2" />
        </radialGradient>
      </defs>

      {/* Outer dashed ring */}
      <g className="reactor-ring reactor-ring--r1" fill="none" stroke="#6fe6ff">
        <circle cx="100" cy="100" r="96" strokeWidth="1" strokeDasharray="2 5" />
        <circle
          cx="100"
          cy="100"
          r="90"
          strokeWidth="1.5"
          strokeDasharray="60 20 8 20"
          opacity="0.7"
        />
      </g>

      {/* Mid segmented ring (counter-rotating) */}
      <g className="reactor-ring reactor-ring--r2" fill="none" stroke="#6fe6ff">
        <circle
          cx="100"
          cy="100"
          r="78"
          strokeWidth="3"
          strokeDasharray="90 40 30 40"
          opacity="0.9"
        />
        <circle cx="100" cy="100" r="70" strokeWidth="1" strokeDasharray="1 6" />
      </g>

      {/* Inner dashed ring */}
      <g className="reactor-ring reactor-ring--r3" fill="none" stroke="#bff5ff">
        <circle cx="100" cy="100" r="58" strokeWidth="2" strokeDasharray="40 25" />
      </g>

      {/* Glowing core */}
      <circle className="reactor-core" cx="100" cy="100" r="46" fill="url(#reactor-core)" />

      {/* Voice waveform */}
      <g className="reactor-wave">
        <rect x="80" y="92" width="5" height="16" rx="2" />
        <rect x="90" y="88" width="5" height="24" rx="2" />
        <rect x="100" y="84" width="5" height="32" rx="2" />
        <rect x="110" y="88" width="5" height="24" rx="2" />
        <rect x="120" y="92" width="5" height="16" rx="2" />
      </g>
    </svg>
  )
}
