interface Props {
  /** True while JARVIS is thinking or speaking; speeds up the core and bars. */
  talking?: boolean
  /** Small fits a toolbar; medium fits the assistant header. */
  size?: 'sm' | 'md'
}

/**
 * Compact arc-reactor orb for JARVIS toolbars. Pure CSS (concentric rings, a
 * breathing core, a rotating scan sweep and a mini waveform) so it costs no
 * JavaScript while it animates. The full-size showpiece lives in JarvisHud.
 */
export function JarvisOrb({ talking = false, size = 'md' }: Props) {
  return (
    <span className={`jv jv-${size}`} data-talking={talking} aria-hidden>
      <span className="jv-ring jv-ring--outer" />
      <span className="jv-ring jv-ring--mid" />
      <span className="jv-core" />
      <span className="jv-wave">
        <i />
        <i />
        <i />
        <i />
        <i />
      </span>
    </span>
  )
}
