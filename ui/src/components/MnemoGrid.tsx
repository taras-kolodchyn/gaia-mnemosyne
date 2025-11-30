export function MnemoGrid({ children }: { children: React.ReactNode }) {
  return <div className="grid gap-6 lg:gap-8 xl:gap-10 lg:grid-cols-3 md:grid-cols-2 grid-cols-1">{children}</div>
}
