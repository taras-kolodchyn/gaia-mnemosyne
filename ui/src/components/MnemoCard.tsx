export function MnemoCard({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <div
      className="rounded-lg border p-4 mnemo-shadow-1 transition-transform transition-shadow duration-200 hover:mnemo-glow-gold hover:scale-[1.01] hover:shadow-lg"
      style={{
        color: 'var(--mnemo-text)',
        background: 'var(--mnemo-bg-2)',
        borderColor: 'rgba(212,166,87,0.15)',
      }}
    >
      <h3 className="text-xl font-semibold mb-2">{title}</h3>
      <div className="text-sm opacity-70">{children}</div>
    </div>
  )
}
