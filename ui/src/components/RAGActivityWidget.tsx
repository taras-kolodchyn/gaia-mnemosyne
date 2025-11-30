import { useAppState } from '../context'

export function RAGActivityWidget() {
  const appState = useAppState()
  const data = appState?.ragHistory ?? []
  return (
    <div className="space-y-2 text-sm">
      {data.length === 0 && <div className="text-xs text-muted-foreground">No recent queries yet.</div>}
      {data.map((q, i) => (
        <div
          key={`${q.query}-${i}`}
          className="rounded border border-gray-700 bg-[var(--mnemo-bg-light)] px-3 py-2"
          style={{ color: 'var(--mnemo-text)' }}
        >
          <div className="flex items-center justify-between">
            <span className="font-semibold">{q.query}</span>
            <span className="text-xs text-muted-foreground">{q.time}</span>
          </div>
        </div>
      ))}
    </div>
  )
}
