export function MnemoSection({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <section className="space-y-2">
      <h2 className="mnemo-h2">{title}</h2>
      <div className="mnemo-body">{children}</div>
    </section>
  )
}
