import type { ReactNode } from 'react'

export function ContentContainer({ children }: { children: ReactNode }) {
  return <div className="w-full max-w-6xl mr-auto">{children}</div>
}
