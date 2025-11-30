export function MnemoLoader() {
  return (
    <div className="flex items-center gap-1 text-[var(--mnemo-accent)]">
      <span className="animate-pulse">●</span>
      <span className="animate-pulse" style={{ animationDelay: '0.15s' }}>
        ●
      </span>
      <span className="animate-pulse" style={{ animationDelay: '0.3s' }}>
        ●
      </span>
    </div>
  )
}
