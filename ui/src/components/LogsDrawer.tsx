import { useEffect, useMemo, useRef, useState } from 'react'
import { useWS } from '../context/WSContext'

const STORAGE_KEY = 'mnemo.logs.height'
const MIN_HEIGHT = 80
const MAX_HEIGHT_RATIO = 0.4
const DEFAULT_HEIGHT = 45

export function LogsDrawer() {
  const { messages } = useWS()
  const [open, setOpen] = useState(false)
  const [height, setHeight] = useState(() => {
    const saved = Number(localStorage.getItem(STORAGE_KEY))
    return Number.isFinite(saved) && saved > MIN_HEIGHT ? saved : DEFAULT_HEIGHT
  })
  const [dragging, setDragging] = useState(false)
  const startYRef = useRef(0)
  const startHeightRef = useRef(0)
  const scrollRef = useRef<HTMLDivElement | null>(null)
  const [autoScroll, setAutoScroll] = useState(true)
  const processedRef = useRef(0)
  const [lines, setLines] = useState<string[]>([])

  useEffect(() => {
    function onMove(e: MouseEvent) {
      if (!dragging) return
      const delta = startYRef.current - e.clientY
      const next = Math.min(
        Math.max(MIN_HEIGHT, startHeightRef.current + delta),
        window.innerHeight * MAX_HEIGHT_RATIO,
      )
      setHeight(next)
    }
    function onUp() {
      if (dragging) {
        setDragging(false)
      }
    }
    window.addEventListener('mousemove', onMove)
    window.addEventListener('mouseup', onUp)
    return () => {
      window.removeEventListener('mousemove', onMove)
      window.removeEventListener('mouseup', onUp)
    }
  }, [dragging])

  useEffect(() => {
    localStorage.setItem(STORAGE_KEY, String(height))
  }, [height])

  const startDrag = (clientY: number) => {
    setDragging(true)
    startYRef.current = clientY
    startHeightRef.current = height
  }

  const logMessages = useMemo(() => messages.filter((m) => m?.event === 'log'), [messages])

  useEffect(() => {
    if (logMessages.length <= processedRef.current) return
    const newSlice = logMessages.slice(processedRef.current)
    processedRef.current = logMessages.length
    const stamped = newSlice.map((m) => {
      const ts =
        typeof m.ts === 'string'
          ? m.ts
          : new Date().toLocaleTimeString('en-GB', {
              hour12: false,
              hour: '2-digit',
              minute: '2-digit',
              second: '2-digit',
            })
      const text =
        typeof m.message === 'string'
          ? m.message
          : typeof m === 'string'
            ? m
            : JSON.stringify(m)
      return `[${ts}] ${text}`
    })
    setLines((prev) => {
      const mut = [...prev]
      for (const line of stamped) {
        const last = mut.length > 0 ? mut[mut.length - 1] : null
        if (last !== null && last === line) continue
        mut.push(line)
      }
      return mut.slice(-500)
    })
    // Always snap to bottom when new logs arrive (unless user scrolled up).
    setTimeout(() => {
      if (scrollRef.current && autoScroll) {
        const el = scrollRef.current
        el.scrollTop = el.scrollHeight
      }
    }, 0)
  }, [logMessages])

  useEffect(() => {
    if (!scrollRef.current || !open) return
    if (autoScroll) {
      const el = scrollRef.current
      el.scrollTop = el.scrollHeight
    }
  }, [lines, open, autoScroll])

  useEffect(() => {
    if (open && scrollRef.current) {
      const el = scrollRef.current
      el.scrollTop = el.scrollHeight
    }
  }, [open])

  const onScroll = () => {
    if (!scrollRef.current) return
    const el = scrollRef.current
    const nearBottom = el.scrollHeight - el.scrollTop - el.clientHeight < 40
    setAutoScroll(nearBottom)
  }

  return (
    <>
      <button
        className="fixed bottom-4 right-4 z-40 rounded-full bg-[var(--mnemo-accent)] text-black px-4 py-2 shadow-lg hover:shadow-xl transition"
        onClick={() => {
          const maxH = window.innerHeight * MAX_HEIGHT_RATIO
          setHeight((h) => Math.min(h, maxH))
          setAutoScroll(true)
          setOpen(true)
        }}
      >
        Logs
      </button>
      {open && (
        <div
          className="fixed bottom-4 left-4 right-4 z-40 max-h-[70vh] bg-[var(--mnemo-bg-3)] border border-[var(--mnemo-bg-4)] rounded-xl shadow-2xl flex flex-col"
          style={{
            height: Math.min(height, window.innerHeight * MAX_HEIGHT_RATIO),
            backgroundColor: 'rgba(13, 27, 42, 0.95)',
          }}
        >
          <div
            className="flex items-center justify-between px-4 py-2 border-b border-[var(--mnemo-bg-4)] text-sm select-none"
            style={{ cursor: 'ns-resize' }}
            onMouseDown={(e) => startDrag(e.clientY)}
          >
            <div className="font-semibold text-[var(--mnemo-text)]">Logs</div>
            <div className="flex items-center gap-2">
              <button
                className="text-[var(--mnemo-text)] opacity-80 hover:opacity-100 text-xs"
                onClick={(e) => {
                  e.stopPropagation()
                  const target = Math.min(window.innerHeight * 0.7, window.innerHeight * MAX_HEIGHT_RATIO)
                  setHeight(target)
                }}
              >
                Expand
              </button>
              <button
                className="text-[var(--mnemo-text)] opacity-80 hover:opacity-100"
                onClick={(e) => {
                  e.stopPropagation()
                  setOpen(false)
                }}
              >
                ✕
              </button>
            </div>
          </div>
          <div
            className="h-[6px] bg-[var(--mnemo-bg-4)] cursor-ns-resize"
            onMouseDown={(e) => startDrag(e.clientY)}
          />
          <div
            ref={scrollRef}
            onScroll={onScroll}
            className="flex-1 overflow-auto p-3 space-y-1 text-[var(--mnemo-text)] text-xs"
            style={{ backgroundColor: 'rgba(13, 27, 42, 0.98)' }}
          >
            {lines.length === 0 && <div className="text-gray-400">No logs yet.</div>}
            {lines.map((line, idx) => {
              const upper = line.toUpperCase()
              const color =
                upper.includes('ERROR')
                  ? 'text-red-400'
                  : upper.includes('WARN')
                    ? 'text-yellow-300'
                    : 'text-[var(--mnemo-text)]'
              return (
                <div key={`${line}-${idx}`} className={`font-mono ${color}`}>
                  {line}
                </div>
              )
            })}
          </div>
        </div>
      )}
    </>
  )
}
