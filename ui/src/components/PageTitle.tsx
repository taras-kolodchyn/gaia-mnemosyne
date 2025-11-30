export function PageTitle({ title, subtitle }: { title: string; subtitle?: string }) {
  return (
    <div className="space-y-1 mt-6">
      <h1 className="mnemo-h1">{title}</h1>
      {subtitle && <p className="mnemo-muted text-sm">{subtitle}</p>}
      <div className="h-[2px] w-16 bg-[var(--mnemo-accent)] mt-2 mb-6 rounded-full"></div>
    </div>
  )
}
