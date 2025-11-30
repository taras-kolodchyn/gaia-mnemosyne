type ChunkDetails = {
  chunkIndex: number
  filePath: string
  text: string
  tags?: string[]
  score?: number
  scores?: { vector?: number; keyword?: number; graph?: number; ontology?: number }
}

type Props = {
  open: boolean
  onClose: () => void
  chunk: ChunkDetails | null
}

export function ChunkDetailsModal({ open, onClose, chunk }: Props) {
  if (!open || !chunk) return null

  const { chunkIndex, filePath, text, tags = [], score, scores } = chunk

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 px-4">
      <div className="w-full max-w-3xl rounded-xl bg-[var(--mnemo-bg-2)] border border-[rgba(212,166,87,0.3)] mnemo-shadow-2 p-6 text-[var(--mnemo-text)] relative">
        <button
          onClick={onClose}
          className="absolute top-3 right-3 text-sm px-3 py-1 rounded border border-[rgba(212,166,87,0.3)] hover:bg-[rgba(212,166,87,0.1)]"
        >
          Close
        </button>
        <div className="text-lg font-semibold mb-1">Chunk #{chunkIndex}</div>
        <div className="text-sm opacity-80 mb-4 break-all">
          File: <span className="text-[var(--mnemo-accent)]">{filePath}</span>
        </div>

        {score !== undefined && (
          <div className="mb-3 text-sm">
            <span className="font-semibold">Score:</span> {score.toFixed(2)}
          </div>
        )}

        {scores && (
          <div className="grid grid-cols-2 gap-2 text-xs mb-4">
            <div>Vector: {scores.vector?.toFixed(2) ?? '-'}</div>
            <div>Keyword: {scores.keyword?.toFixed(2) ?? '-'}</div>
            <div>Graph: {scores.graph?.toFixed(2) ?? '-'}</div>
            <div>Ontology: {scores.ontology?.toFixed(2) ?? '-'}</div>
          </div>
        )}

        {tags.length > 0 && (
          <div className="flex flex-wrap gap-2 mb-4">
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

        <div className="text-sm leading-relaxed whitespace-pre-wrap bg-[var(--mnemo-bg-light)]/60 rounded p-3 border border-[rgba(212,166,87,0.2)] max-h-[55vh] overflow-auto">
          {text}
        </div>
      </div>
    </div>
  )
}
