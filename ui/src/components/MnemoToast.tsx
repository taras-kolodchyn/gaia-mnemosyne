import { useAppState } from '../context'

export function MnemoToast() {
  const app = useAppState()
  if (!app) return null

  return (
    <div className="fixed top-4 right-4 z-50 space-y-2 w-72">
      {app.toasts.map((toast) => (
        <div
          key={toast.id}
          className="rounded border border-[rgba(212,166,87,0.4)] bg-[var(--mnemo-bg-2)] text-[var(--mnemo-text)] px-4 py-3 shadow-lg mnemo-glow-gold"
        >
          <div className="flex justify-between items-center gap-2">
            <span className="text-sm">{toast.message}</span>
            <button
              className="text-xs opacity-70 hover:opacity-100"
              onClick={() => app.removeToast(toast.id)}
            >
              ×
            </button>
          </div>
        </div>
      ))}
    </div>
  )
}
