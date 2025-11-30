import { useMemo } from 'react'
import { MnemoModal } from './MnemoModal'
import { MnemoStatusBadge } from './MnemoStatusBadge'
import { useWS } from '../context/WSContext'
import { mnemoPost } from '../api/client'
import { useState } from 'react'
import { useEffect, useRef } from 'react'
import { formatTimestamp } from '../utils/date'
import { useAppState } from '../context/AppStateContext'

type TimelineStatus = 'pending' | 'running' | 'success' | 'failed'

const steps: TimelineStatus[] = ['pending', 'running', 'success']

function Timeline({ status }: { status: TimelineStatus }) {
  const statusIndex = steps.indexOf(status === 'failed' ? 'running' : status)

  return (
    <div className="flex items-center gap-2">
      {steps.map((step, idx) => {
        const done = idx < statusIndex
        const current = idx === statusIndex
        const isFailed = status === 'failed' && step === 'success'
        const color = isFailed
          ? 'bg-red-500'
          : done
            ? 'bg-green-500'
            : current && step === 'running'
              ? 'bg-blue-400 animate-pulse'
              : current
                ? 'bg-yellow-400'
                : 'bg-gray-600'
        const lineColor = isFailed ? 'border-red-500' : done ? 'border-green-500' : 'border-gray-700'

        const dot = (
          <div
            className={`h-3 w-3 rounded-full transition-all ${color}`}
            title={step}
          />
        )

        return (
          <div key={step} className="flex items-center gap-2">
            {dot}
            {idx < steps.length - 1 && (
              <div className={`w-10 border-t border-dashed ${lineColor} transition-all`} />
            )}
          </div>
        )
      })}
    </div>
  )
}

export function JobDetailsModal({
  open,
  onClose,
  job,
}: {
  open: boolean
  onClose: () => void
  job: {
    id: string
    job_type: string
    status: 'pending' | 'running' | 'success' | 'failed'
    created_at: string
    updated_at: string
  } | null
}) {
  const [retrying, setRetrying] = useState(false)
  const { logs, logsByJob, messages, ingestionSteps } = useWS()
  const { addToast } = useAppState()
  const jobLogs = useMemo(() => {
    if (!job) return []
    // Prefer per-job store, fallback to filtering global logs
    const direct = logsByJob[job.id] ?? []
    if (direct.length > 0) return direct
    return logs
      .filter((m) => m?.event === 'log' && (!m.job_id || m.job_id === job.id))
      .map((m) => ({
        ts: m.ts ?? new Date().toISOString(),
        message: m.message ?? '',
      }))
  }, [logs, logsByJob, job])
  const logsEndRef = useRef<HTMLDivElement | null>(null)
  const logContainerRef = useRef<HTMLDivElement | null>(null)
  const [autoScroll, setAutoScroll] = useState(true)

  useEffect(() => {
    if (autoScroll && logsEndRef.current) {
      logsEndRef.current.scrollIntoView({ behavior: 'smooth' })
    }
  }, [jobLogs.length, autoScroll])

  // Detect if user manually scrolls up; if so, disable autoscroll until bottom reached again.
  useEffect(() => {
    const el = logContainerRef.current
    if (!el) return
    const handler = () => {
      const atBottom = el.scrollTop + el.clientHeight >= el.scrollHeight - 10
      setAutoScroll(atBottom)
    }
    el.addEventListener('scroll', handler)
    return () => el.removeEventListener('scroll', handler)
  }, [])

  const errorsFromWs = useMemo(() => {
    if (!job) return []
    return messages
      .filter((m) => m?.event === 'ingest_error_summary' && (!m.job_id || m.job_id === job.id))
      .flatMap((m) => (Array.isArray(m.errors) ? m.errors : []))
  }, [messages, job])

  const errors = [
    ...errorsFromWs,
    ...jobLogs
      .filter((l) => l.message.toLowerCase().includes('fail'))
      .map((l) => l.message),
  ]

  const stepHints = [
    { key: 'start', label: 'Start', keyword: 'starting filesystem ingestion' },
    { key: 'fingerprints', label: 'Fingerprints', keyword: 'fingerprint' },
    { key: 'chunking', label: 'Chunking', keyword: 'chunk' },
    { key: 'ontology', label: 'Ontology', keyword: 'ontology' },
    { key: 'embeddings', label: 'Embeddings', keyword: 'embedding' },
    { key: 'vector_upsert', label: 'Vector upsert', keyword: 'qdrant' },
    { key: 'graph_upsert', label: 'Graph upsert', keyword: 'graph upsert' },
    { key: 'completed', label: 'Completed', keyword: 'completed' },
  ]

  const stepStates = useMemo(() => {
    if (!job) return {}
    const map: Record<string, string> = {}
    // Prefer stored step states from WSContext.ingestionSteps
    const fromCtx = ingestionSteps[job.id] ?? {}
    Object.keys(fromCtx).forEach((k) => {
      map[k] = fromCtx[k]
    })
    // Also fold in any adhoc ingest_step messages that might have arrived before context sync.
    messages
      .filter((m) => m?.event === 'ingest_step' && (!m.job_id || m.job_id === job.id))
      .forEach((m) => {
        const key = (m.step as string | undefined)?.toLowerCase() || ''
        if (key) {
          map[key] = (m.status as string | undefined)?.toLowerCase() || map[key] || 'pending'
        }
      })
    return map
  }, [messages, job, ingestionSteps])

  const stepStatus = stepHints.map((s) => {
    const statusFromEvent = stepStates[s.key]
    let state = statusFromEvent || 'pending'
    if (state === 'done') state = 'success'
    if (state === 'panic') state = 'failed'
    if (!statusFromEvent && job?.status === 'failed') state = 'failed'
    if (!statusFromEvent && job?.status === 'success') state = 'success'
    return { ...s, state }
  })

  const displayStatus: TimelineStatus = useMemo(() => {
    const values = Object.values(stepStates).map((s) => s?.toLowerCase?.() || s)
    if (values.includes('failed') || values.includes('panic')) return 'failed'
    if (stepStates['completed'] === 'done') return 'success'
    if (values.includes('running')) return 'running'
    return job?.status ?? 'pending'
  }, [stepStates, job])

  const retry = async () => {
    if (!job) return
    try {
      setRetrying(true)
      await mnemoPost('/v1/jobs/run', { job_id: job.id })
    } catch (e) {
      console.error('Retry failed', e)
    } finally {
      setRetrying(false)
    }
  }

  const copyId = async () => {
    if (!job) return
    try {
      await navigator.clipboard.writeText(job.id)
      addToast?.('Job ID copied')
    } catch (e) {
      console.error('copy failed', e)
    }
  }

  if (!job) return null
  return (
    <MnemoModal open={open} onClose={onClose} title="Job Details">
      <div className="rounded-2xl bg-[var(--mnemo-bg-2)] border border-[rgba(212,166,87,0.15)] p-6 text-sm space-y-4 mnemo-shadow-2 w-full max-w-6xl">
        <div className="flex items-start justify-between pb-3 border-b border-white/5">
          <div className="space-y-1">
            <div className="text-xl font-semibold text-[var(--mnemo-text)]">Job Details</div>
            <div className="text-xs text-gray-400">Ingestion metadata and live logs</div>
          </div>
          <button
            className="rounded px-3 py-1 text-xs border border-[rgba(212,166,87,0.3)] bg-[var(--mnemo-bg-2)] text-[var(--mnemo-text)] hover:mnemo-glow-gold"
            onClick={onClose}
          >
            Close
          </button>
        </div>

        <div className="border-b border-white/5 pb-3 space-y-2">
          <div className="text-xs text-gray-400">Job ID</div>
          <div className="flex items-center gap-2">
            <span className="font-mono text-[var(--mnemo-text)] break-all text-xs">{job.id}</span>
            <button
              className="text-[var(--mnemo-accent)] text-xxs underline cursor-pointer hover:opacity-80"
              onClick={copyId}
              title="Copy job id"
            >
              copy
            </button>
          </div>
        </div>

        <div className="grid gap-4 md:grid-cols-2">
          <div className="space-y-4">
            <div className="space-y-2">
              <div className="text-sm font-semibold">Overview</div>
              <div className="flex items-center justify-between">
                <div>
                  <div className="text-xs text-gray-400">Job Type</div>
                  <div className="font-semibold">{job.job_type}</div>
                </div>
                <div className="text-sm font-semibold">
                  {displayStatus === 'pending'
                    ? 'Pending'
                    : displayStatus === 'running'
                      ? 'Running'
                      : displayStatus === 'success'
                        ? 'Success'
                        : 'Failed'}
                </div>
              </div>
              <div className="border-t border-white/5 pt-3 space-y-2">
                <div className="text-xs text-gray-400">Timeline</div>
                <Timeline status={displayStatus} />
              </div>
            </div>

            <div className="space-y-2 border-t border-white/5 pt-3">
              <div className="text-sm font-semibold">Timestamps</div>
              <div className="grid grid-cols-2 gap-3">
                <div>
                  <div className="text-xs text-gray-400">Created</div>
                  <div className="text-[var(--mnemo-text)]">{formatTimestamp(job.created_at)}</div>
                </div>
                <div>
                  <div className="text-xs text-gray-400">Updated</div>
                  <div className="text-[var(--mnemo-text)]">{formatTimestamp(job.updated_at)}</div>
                </div>
              </div>
            </div>

            <div className="space-y-2 border-t border-white/5 pt-3">
              <div className="text-sm font-semibold">Ingestion Steps</div>
              <div className="space-y-1">
                {stepStatus.map((s) => {
                  const accent =
                    s.key === 'ontology'
                      ? 'text-blue-300'
                      : s.state === 'success'
                        ? 'text-green-400'
                        : s.state === 'failed'
                          ? 'text-red-400'
                          : s.state === 'running'
                            ? 'text-blue-300'
                            : 'text-gray-400'
                  return (
                  <div
                    key={s.key}
                    className="flex items-center justify-between rounded border border-white/5 bg-white/5 px-2 py-1"
                  >
                    <span>{s.label}</span>
                    <span
                      className={`text-xs ${accent}`}
                    >
                      {s.state === 'success'
                        ? 'done'
                        : s.state === 'failed'
                          ? 'failed'
                          : s.state === 'running'
                            ? 'running'
                            : 'pending'}
                    </span>
                  </div>
                  )
                })}
              </div>
            </div>

            {errors.length > 0 && (
              <div className="rounded border border-red-500/40 bg-red-500/10 p-3 text-red-200">
                <div className="font-semibold">Errors</div>
                <ul className="mt-1 list-disc space-y-1 pl-4">
                  {errors.map((e, idx) => (
                    <li key={idx}>{typeof e === 'string' ? e : (e as any)?.message ?? `${e}`}</li>
                  ))}
                </ul>
              </div>
            )}
          </div>

          <div className="rounded-2xl border border-white/5 bg-white/5 p-3 flex flex-col">
            <div className="mb-2 flex items-center justify-between">
              <div className="text-sm font-semibold">Logs</div>
              <span className="text-xs text-gray-400">live</span>
            </div>
            {jobLogs.length === 0 ? (
              <div className="text-xs text-gray-400">No logs for this job yet.</div>
            ) : (
              <div
                ref={logContainerRef}
                className="max-h-72 space-y-1 overflow-y-auto text-xs pr-1"
              >
                {jobLogs.map((log, idx) => (
                  <div
                    key={`${log.ts}-${idx}`}
                    className="flex items-start gap-2 rounded bg-black/20 px-2 py-1"
                  >
                    <span className="text-gray-400">{formatTimestamp(log.ts)}</span>
                    <span className="text-gray-100">{log.message}</span>
                  </div>
                ))}
                <div ref={logsEndRef} />
              </div>
            )}
          </div>
        </div>

        <div className="pt-2 flex justify-end gap-2 border-t border-white/5">
          {job.status === 'failed' && (
            <button
              className="rounded px-4 py-2 text-black"
              style={{ background: 'var(--mnemo-accent)' }}
              onClick={retry}
              disabled={retrying}
            >
              {retrying ? 'Retrying…' : 'Retry ingestion'}
            </button>
          )}
          <button
            className="rounded px-4 py-2 text-black"
            style={{ background: 'var(--mnemo-accent)' }}
            onClick={onClose}
          >
            Close
          </button>
        </div>
      </div>
    </MnemoModal>
  )
}
