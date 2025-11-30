type Props = {
  chunkIndex: number
  filePath: string
  text: string
  tags?: string[]
  score?: number
  onView?: () => void
}

export function MnemoChunkCard({ chunkIndex, filePath, text, tags = [], score, onView }: Props) {
  const preview =
    text.length > 300 ? `${text.slice(0, 300).trimEnd()}…` : text

  const scoreBg =
    score !== undefined
      ? score > 0.8
        ? 'bg-green-900/30'
        : score > 0.6
          ? 'bg-green-700/20'
          : score > 0.4
            ? 'bg-green-500/10'
            : ''
      : ''

  return (
    <div
      className={`border border-[rgba(212,166,87,0.25)] rounded-xl bg-[var(--mnemo-bg-2)] p-4 mnemo-shadow-1 hover:mnemo-glow-gold transition-all hover:scale-[1.01] ${scoreBg}`}
    >
      <div className="flex items-center justify-between text-xs text-[var(--mnemo-text)]/70 mb-2">
        <span className="font-semibold text-[var(--mnemo-text)]">Chunk #{chunkIndex}</span>
        {score !== undefined && (
          <span className="px-2 py-1 rounded bg-[rgba(212,166,87,0.15)] text-[var(--mnemo-text)] text-xs">
            Score: {score.toFixed(2)}
          </span>
        )}
      </div>
      {score !== undefined && (
        <div className="w-full h-1 rounded bg-black/30 mb-2">
          <div
            className="h-1 rounded bg-[var(--mnemo-accent)] transition-all"
            style={{ width: `${Math.min(100, Math.max(0, score * 100))}%` }}
          />
        </div>
      )}
      <div className="text-[var(--mnemo-text)] text-sm mb-2 break-words">
        <span className="text-[var(--mnemo-accent)]/80 text-xs">File:</span> {filePath}
      </div>
      <p className="text-sm text-[var(--mnemo-text)]/90 mb-3 whitespace-pre-wrap break-words">
        {preview || 'No content'}
      </p>
      {tags.length > 0 && (
        <div className="flex flex-wrap gap-2 mb-3">
          {tags.map((tag) => (
            <span
              key={tag}
              className="text-xs px-2 py-1 rounded-full bg-[rgba(212,166,87,0.15)] text-[var(--mnemo-text)]/80 border border-[rgba(212,166,87,0.25)]"
            >
              {tag}
            </span>
          ))}
        </div>
      )}
      <div className="flex justify-end">
        <button
          onClick={onView}
          className="px-3 py-1 text-xs rounded border border-[rgba(212,166,87,0.5)] text-[var(--mnemo-text)] hover:bg-[rgba(212,166,87,0.15)] transition-colors"
        >
          View more
        </button>
      </div>
    </div>
  )
}
