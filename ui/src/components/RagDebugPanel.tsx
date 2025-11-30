import { MnemoCard } from './MnemoCard'
import { MnemoSkeleton } from './MnemoSkeleton'

type Candidate = {
  chunk: string
  vector_score: number
  keyword_score: number
  graph_score: number
  knowledge_score?: number
  final_score: number
}

type Props = {
  loading: boolean
  candidates: Candidate[]
}

export function RagDebugPanel({ loading, candidates }: Props) {
  if (loading) {
    return (
      <div className="space-y-3">
        <MnemoSkeleton />
        <MnemoSkeleton />
        <MnemoSkeleton />
      </div>
    )
  }

  if (!candidates || candidates.length === 0) {
    return <div className="text-sm opacity-70">No debug data available yet.</div>
  }

  const bgForScore = (score: number) => {
    if (score > 0.8) return 'bg-green-600/30'
    if (score > 0.5) return 'bg-green-900/30'
    if (score > 0) return 'bg-green-950/10'
    return ''
  }

  return (
    <div className="grid gap-4 md:grid-cols-2 grid-cols-1">
      {candidates.map((c, idx) => (
        <MnemoCard key={idx} title={`Chunk ${idx + 1}`}>
          <div className="space-y-2 text-sm">
            <div className={`opacity-80 rounded p-2 transition-colors ${bgForScore(c.final_score)}`}>
              {c.chunk.length > 120 ? `${c.chunk.slice(0, 120)}…` : c.chunk}
            </div>
            <div className="grid grid-cols-2 gap-2 text-xs opacity-80">
              <div>Vector: {c.vector_score.toFixed(3)}</div>
              <div>Keyword: {c.keyword_score.toFixed(3)}</div>
              <div>Graph: {c.graph_score.toFixed(3)}</div>
              {c.knowledge_score !== undefined && <div>Knowledge: {c.knowledge_score.toFixed(3)}</div>}
              <div className="font-semibold text-[var(--mnemo-text)]">Final: {c.final_score.toFixed(3)}</div>
            </div>
            <div className="w-full h-1.5 rounded bg-black/30 overflow-hidden">
              <div
                className="h-1.5 rounded bg-[var(--mnemo-accent)] transition-all"
                style={{ width: `${Math.min(100, Math.max(0, c.final_score * 100))}%` }}
              />
            </div>
          </div>
        </MnemoCard>
      ))}
    </div>
  )
}
