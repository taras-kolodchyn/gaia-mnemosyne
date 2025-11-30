import { createContext, useContext, useEffect, useRef, useState } from "react"

type WSContextValue = {
  ws: WebSocket | null
  connected: boolean
  messages: any[]
  logs: any[]
  logsByJob: Record<string, { ts: string; message: string }[]>
  ingestionSteps: Record<string, Record<string, string>>
  addLocalLog: (text: string) => void
  heartbeat: number
  pipelineCrashed: boolean
}

const WSContext = createContext<WSContextValue | null>(null)

export function WSProvider({ children }: { children: React.ReactNode }) {
  const wsRef = useRef<WebSocket | null>(null)
  const [connected, setConnected] = useState(false)
  const [messages, setMessages] = useState<any[]>([])
  const [logs, setLogs] = useState<any[]>([])
  const [logsByJob, setLogsByJob] = useState<Record<string, { ts: string; message: string }[]>>({})
  const [ingestionSteps, setIngestionSteps] = useState<Record<string, Record<string, string>>>({})
  const [heartbeat, setHeartbeat] = useState<number>(Date.now())
  const [pipelineCrashed, setPipelineCrashed] = useState(false)

  const addLocalLog = (text: string) => {
    setMessages((prev) => [
      ...prev,
      { event: 'log', message: text, local: true, ts: new Date().toISOString() },
    ])
  }

  useEffect(() => {
    const url = `${import.meta.env.VITE_API_WS_URL || 'ws://localhost:7700'}/ws/all`
    const ws = new WebSocket(url)
    wsRef.current = ws

    ws.onopen = () => setConnected(true)
    ws.onclose = () => setConnected(false)
    ws.onerror = () => setConnected(false)
    ws.onmessage = (event) => {
      try {
        const msg = JSON.parse(event.data)
        const enriched = {
          ...msg,
          ts: msg.ts || new Date().toISOString(),
        }
        const next: any[] = [enriched]

        // If pipeline crashes or a panic step is reported, synthesize failure events to
        // make UI mark the job and steps as failed.
        const isPanicStep =
          enriched.event === 'ingest_step' && typeof enriched.step === 'string' && enriched.step === 'panic'
        const isPipelineFailed = enriched.event === 'pipeline_failed'
        if ((isPanicStep || isPipelineFailed) && enriched.job_id) {
          const steps = [
            'start',
            'fingerprints',
            'chunking',
            'ontology',
            'embeddings',
            'vector_upsert',
            'graph_upsert',
            'completed',
          ]
          // Mark job as failed so UI can disable Run Now and show failure.
          next.push({
            event: 'job_update',
            job_id: enriched.job_id,
            status: 'failed',
            progress: 0,
            ts: enriched.ts,
          })
          // Mark all steps as failed to keep timelines in sync.
          steps.forEach((step) => {
            next.push({
              event: 'ingest_step',
              job_id: enriched.job_id,
              step,
              status: 'failed',
              ts: enriched.ts,
            })
          })
          setPipelineCrashed(true)
        }

        setMessages((prev) => [...prev, ...next])
        setHeartbeat(Date.now())
        next.forEach((m) => {
          if (m.event === 'log' || m.event === 'ingest_log') {
            const normalized = {
              event: 'log',
              job_id: m.job_id,
              message: m.message ?? '',
              ts: m.ts ?? new Date().toISOString(),
            }
            setLogs((prev) => [...prev, normalized])
            if (normalized.job_id) {
              setLogsByJob((prev) => {
                const current = prev[normalized.job_id] ?? []
                return {
                  ...prev,
                  [normalized.job_id]: [...current, { ts: normalized.ts, message: normalized.message }],
                }
              })
            }
          }
          if (m.event === 'ingest_step' && m.job_id && m.step) {
            setIngestionSteps((prev) => {
              const current = prev[m.job_id] ?? {}
              return {
                ...prev,
                [m.job_id]: {
                  ...current,
                  [m.step]: m.status ?? current[m.step] ?? 'pending',
                },
              }
            })
          }
        })
      } catch (_) {
        // ignore parse errors
      }
    }

    // Keep WS alive until full page reload.
    return () => {}
  }, [])

  return (
    <WSContext.Provider
      value={{
        ws: wsRef.current,
        connected,
        messages,
        logs,
        logsByJob,
        ingestionSteps,
        addLocalLog,
        heartbeat,
        pipelineCrashed,
      }}
    >
      {children}
    </WSContext.Provider>
  )
}

export function useWS() {
  const ctx = useContext(WSContext)
  if (!ctx) {
    throw new Error('useWS must be used within WSProvider')
  }
  return ctx
}
