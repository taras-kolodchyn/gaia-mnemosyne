import { useEffect, useState } from 'react'
import { NavLink, useLocation } from 'react-router-dom'
import { MnemoLogo } from './MnemoLogo'
import { useWS } from '../context/WSContext'

const links = [
  { to: '/', label: 'Dashboard' },
  { to: '/sources', label: 'Sources' },
  { to: '/rag', label: 'RAG Playground' },
  { to: '/status', label: 'System Status' },
  { to: '/pipeline', label: 'Pipeline Monitor' },
  { to: '/graph', label: 'Graph Explorer' },
]

export function SideNav() {
  const location = useLocation()
  const { connected, messages } = useWS()
  const [rtt, setRtt] = useState<number | null>(null)

  useEffect(() => {
    const reversed = [...messages].reverse()
    const lastPong = reversed.find((m) => m.event === 'pong' && typeof (m as any).ts === 'number')
    if (lastPong && typeof (lastPong as any).rtt === 'number') {
      setRtt((lastPong as any).rtt)
    }
  }, [messages])

  return (
    <aside
      className="h-screen w-60 border-r border-gray-800"
      style={{ background: 'var(--mnemo-bg-1)', color: 'var(--mnemo-text)' }}
    >
      <div className="px-5 py-6 space-y-3">
        <MnemoLogo />
        <div className="flex items-center gap-2 text-xs text-[var(--mnemo-text)] opacity-80">
          <span
            className={`h-2 w-2 rounded-full ${connected ? 'bg-green-400' : 'bg-gray-500'}`}
            title={connected ? 'WebSocket connected' : 'WebSocket disconnected'}
          ></span>
          <span>WS Status</span>
          {rtt !== null && <span className="ml-auto">RTT: {Math.round(rtt)} ms</span>}
        </div>
      </div>
      <nav className="px-3 space-y-1 text-sm">
        {links.map((link) => (
          <NavLink
            key={link.to}
            to={link.to}
            className={({ isActive }) => {
              const active =
                isActive || location.pathname === link.to || location.pathname.startsWith(`${link.to}/`)
              return [
                'block rounded px-3 py-2 transition',
                active
                  ? 'font-bold text-[var(--mnemo-accent)]'
                  : 'opacity-80 text-[var(--mnemo-text)] hover:text-[var(--mnemo-accent)]',
              ].join(' ')
            }}
          >
            {link.label}
          </NavLink>
        ))}
      </nav>
    </aside>
  )
}
