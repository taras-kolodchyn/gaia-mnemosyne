import { useEffect, useState } from 'react'

export function SystemBanner({ allUp, anyDown }: { allUp: boolean; anyDown: boolean }) {
  const [wsDisconnected, setWsDisconnected] = useState(false)

  useEffect(() => {
    const handler = (e: Event) => {
      const detail = (e as CustomEvent).detail
      if (detail && typeof detail.connected === 'boolean') {
        setWsDisconnected(!detail.connected)
      }
    }
    window.addEventListener('mnemo-ws-status', handler as EventListener)
    return () => window.removeEventListener('mnemo-ws-status', handler as EventListener)
  }, [])

  if (wsDisconnected) {
    return (
      <div className="mb-4 rounded border border-yellow-700/40 bg-yellow-900/30 px-4 py-3 text-yellow-200 shadow">
        ⚠ Connection to live updates lost
      </div>
    )
  }

  if (allUp) {
    return (
      <div className="mb-4 flex items-center justify-between rounded-lg bg-green-900/30 px-4 py-3 text-green-200 shadow">
        <span className="text-lg font-semibold">✓ All systems operational</span>
      </div>
    )
  }

  if (anyDown) {
    return (
      <div className="mb-6 rounded border border-red-700/40 bg-red-900/40 p-4 text-red-200">
        ⚠ System degraded: Some services are unavailable
      </div>
    )
  }

  return null
}
