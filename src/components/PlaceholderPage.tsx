interface Props {
  title: string
  body: string
}

/** Screen for features that are planned but not implemented yet (Phases 3+). */
export function PlaceholderPage({ title, body }: Props) {
  return (
    <div>
      <h1>{title}</h1>
      <p className="sub">{body}</p>
      <div
        style={{
          border: '1px dashed var(--color-border)',
          borderRadius: 'var(--radius-lg)',
          padding: '3rem 1.5rem',
          textAlign: 'center',
          color: 'var(--color-muted)',
        }}
      >
        This screen arrives in a later phase. Until then, use the pages above.
      </div>
    </div>
  )
}
