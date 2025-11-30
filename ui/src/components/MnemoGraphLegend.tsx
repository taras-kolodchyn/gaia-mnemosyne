export function MnemoGraphLegend() {
  const items = [
    { label: 'Repo', color: 'bg-blue-500 border-blue-300' },
    { label: 'File', color: 'bg-green-500 border-green-300' },
    { label: 'Chunk', color: 'bg-yellow-400 border-yellow-300' },
  ]

  return (
    <div className="flex items-center gap-3 text-xs text-[var(--mnemo-text)]">
      {items.map((i) => (
        <div key={i.label} className="flex items-center gap-2">
          <span className={`h-3 w-3 rounded-full border ${i.color}`}></span>
          <span>{i.label}</span>
        </div>
      ))}
    </div>
  )
}
