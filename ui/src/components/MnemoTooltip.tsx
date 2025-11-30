export function MnemoTooltip({ text, children }: { text: string; children: React.ReactNode }) {
  return (
    <div className="group relative inline-block">
      {children}
      <div className="absolute left-1/2 z-50 mt-1 hidden -translate-x-1/2 whitespace-nowrap rounded bg-black/80 px-2 py-1 text-xs text-white shadow group-hover:block">
        {text}
      </div>
    </div>
  )
}
