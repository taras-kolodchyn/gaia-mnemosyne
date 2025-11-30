import type { PropsWithChildren } from 'react'
import { SideNav } from './SideNav'

export function Layout({ children }: PropsWithChildren) {
  return (
    <div className="min-h-screen" style={{ background: 'var(--mnemo-bg)', color: 'var(--mnemo-text)' }}>
      <div className="grid min-h-screen" style={{ gridTemplateColumns: '240px 1fr' }}>
        <SideNav />
        <main className="min-h-screen">
          <div className="mx-auto w-full max-w-[1800px] px-8" style={{ paddingBottom: '1.5rem' }}>
            <div className="space-y-6 pt-6">{children}</div>
          </div>
        </main>
      </div>
    </div>
  )
}
