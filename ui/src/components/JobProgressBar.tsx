export function JobProgressBar({ percent }: { percent: number }) {
  return (
    <div className="w-full h-2 rounded bg-black/30 overflow-hidden">
      <div
        className="h-2 rounded bg-[var(--mnemo-accent)] transition-all"
        style={{ width: `${Math.min(100, Math.max(0, percent))}%` }}
      />
    </div>
  )
}
