export function MnemoTabs({ tabs, active, onChange }: { tabs: string[]; active: string; onChange: (t: string) => void }) {
  return (
    <div className="flex gap-2">
      {tabs.map((t) => (
        <button
          key={t}
          onClick={() => onChange(t)}
          className={`px-4 py-2 rounded border-b-2 ${
            active === t
              ? 'bg-[var(--mnemo-accent)] text-black border-[var(--mnemo-accent)]'
              : 'bg-[var(--mnemo-bg-light)] text-[var(--mnemo-text)] border-transparent'
          }`}
        >
          {t}
        </button>
      ))}
    </div>
  )
}
