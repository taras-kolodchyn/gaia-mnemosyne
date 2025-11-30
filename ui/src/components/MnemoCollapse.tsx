import { useState } from 'react'

export function MnemoCollapse({ title, children }: { title: string; children: React.ReactNode }) {
  const [open, setOpen] = useState(true)

  return (
    <div className="border border-gray-700 rounded-lg p-4" style={{ color: 'var(--mnemo-text)' }}>
      <button
        type="button"
        className="flex items-center gap-2 text-[var(--mnemo-text)] font-semibold"
        onClick={() => setOpen((o) => !o)}
      >
        <span>{title}</span>
        <span>{open ? '▾' : '▸'}</span>
      </button>
      {open && <div className="mt-4">{children}</div>}
    </div>
  )
}
