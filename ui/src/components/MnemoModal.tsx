export function MnemoModal({
  open,
  onClose,
  title,
  children,
}: {
  open: boolean
  onClose: () => void
  title: string
  children: React.ReactNode
}) {
  if (!open) return null

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60">
      <div className="w-full max-w-lg rounded-lg border border-gray-700 bg-[var(--mnemo-bg-light)] p-6 shadow-xl" style={{ color: 'var(--mnemo-text)' }}>
        <div className="mb-4 flex items-center justify-between">
          <h3 className="text-lg font-semibold">{title}</h3>
          <button
            onClick={onClose}
            className="rounded px-2 py-1 text-sm transition hover:bg-gray-700"
            style={{ color: 'var(--mnemo-text)' }}
          >
            Close
          </button>
        </div>
        <div className="text-sm text-muted-foreground">{children}</div>
      </div>
    </div>
  )
}
