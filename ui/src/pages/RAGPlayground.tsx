import { useEffect, useState } from 'react'
import { useAppState } from '../context'
import { mnemoPost } from '../api/client'
import { MnemoCard } from '../components/MnemoCard'
import { PageTitle } from '../components/PageTitle'
import { MnemoSkeleton } from '../components/MnemoSkeleton'
import { MnemoTabs } from '../components/MnemoTabs'
import { ContentContainer } from '../components/ContentContainer'
import { mnemoGet } from '../api/client'
import { RagDebugPanel } from '../components/RagDebugPanel'
import { MnemoCollapse } from '../components/MnemoCollapse'
import { MnemoLoader } from '../components/MnemoLoader'
import { useWS } from '../context/WSContext'
import { MnemoGrid } from '../components/MnemoGrid'
import { MnemoChunkCard } from '../components/MnemoChunkCard'

export function RAGPlayground() {
  const [query, setQuery] = useState('')
  const appState = useAppState()
  const [activeTab, setActiveTab] = useState<'Query' | 'Settings'>('Query')
  const [rankingProfile, setRankingProfile] = useState('default')
  const [expandGraph, setExpandGraph] = useState(false)
  const [showRaw, setShowRaw] = useState(false)
  const [metadata, setMetadata] = useState<{ vector_hits: number; graph_depth: number; response_time_ms: number } | null>(null)
  const [showDebug, setShowDebug] = useState(false)
  const [debugResults, setDebugResults] = useState<any[]>([])
  const [debugLoading, setDebugLoading] = useState(false)
  const [ragProcessing, setRagProcessing] = useState(false)
  const { messages } = useWS()

  async function loadDebug(q: string) {
    if (!showDebug || !q.trim()) return
    setDebugLoading(true)
    try {
      const debug = await mnemoPost('/v1/rag/debug', { query: q })
      setDebugResults(debug?.candidates ?? [])
    } finally {
      setDebugLoading(false)
    }
  }

  async function runQuery() {
    try {
      appState?.setLoading(true)
      const data = await mnemoPost('/v1/rag/query', { query })
      appState?.setRagResponse(data)
      appState?.addRagHistoryEntry(query)
      const meta = await mnemoGet('/v1/rag/metadata')
      setMetadata(meta)
      await loadDebug(query)
    } finally {
      appState?.setLoading(false)
    }
  }

  useEffect(() => {
    const last = messages[messages.length - 1]
    if (!last) return
    if (last.event === 'rag_processing') {
      setRagProcessing(true)
    }
    if (last.event === 'rag_done') {
      setRagProcessing(false)
    }
  }, [messages])

  return (
    <ContentContainer>
      <section className="space-y-6 mb-10">
        <PageTitle title="RAG Playground" />
        <MnemoCard title="RAG Playground">
          <div className="space-y-4">
            <MnemoTabs tabs={['Query', 'Settings']} active={activeTab} onChange={(t) => setActiveTab(t as any)} />

            {activeTab === 'Query' ? (
              <>
                <div className="grid gap-6 xl:grid-cols-[1.3fr_1fr_0.8fr] lg:grid-cols-[1fr_1fr] grid-cols-1 items-start">
                  <div className="space-y-3">
                    <p className="text-sm text-muted-foreground">
                      Run ad-hoc RAG queries against Gaia Mnemosyne.
                    </p>
                    <input
                      type="text"
                      value={query}
                      onChange={(e) => setQuery(e.target.value)}
                      className="w-full rounded border p-3"
                      style={{ background: 'var(--mnemo-bg-light)', color: 'var(--mnemo-text)' }}
                      placeholder="Enter your query..."
                    />
                    <button
                      className="mt-2 rounded px-4 py-2 text-white"
                      style={{ background: 'var(--mnemo-accent)' }}
                      onClick={runQuery}
                    >
                      Run Query
                    </button>
                    <label className="flex items-center gap-2 text-sm font-semibold">
                      <input
                        type="checkbox"
                        checked={showDebug}
                        onChange={async (e) => {
                          setShowDebug(e.target.checked)
                          if (e.target.checked) {
                            await loadDebug(query)
                          } else {
                            setDebugResults([])
                          }
                        }}
                      />
                      Show Debug Info
                    </label>
                  </div>
                  <div className="rounded border border-gray-700 bg-[var(--mnemo-bg-light)] p-4 shadow-inner">
                    <h3 className="text-lg font-semibold" style={{ color: 'var(--mnemo-text)' }}>
                      Response
                    </h3>
                    {ragProcessing && (
                      <div className="mt-2 flex items-center gap-2 text-sm text-[var(--mnemo-text)]">
                        <MnemoLoader />
                        <span>Processing query…</span>
                      </div>
                    )}
                    {appState?.ragResponse ? (
                      <div className="mt-2 flex justify-end">
                        <button
                          className="rounded px-3 py-1 text-xs border border-[rgba(212,166,87,0.3)] bg-[var(--mnemo-bg-2)] text-[var(--mnemo-text)] mnemo-shadow-1 hover:mnemo-glow-gold"
                          onClick={() => {
                            const text = JSON.stringify(appState?.ragResponse ?? {}, null, 2)
                            if (text) {
                              navigator.clipboard.writeText(text)
                              // lightweight toast placeholder
                          window.alert('Copied RAG response to clipboard')
                        }
                      }}
                    >
                      Copy JSON
                    </button>
                    </div>
                  ) : null}
                  {Boolean(appState?.ragResponse) ? (
                    <div className="mt-4 space-y-3">
                      <MnemoCollapse title="Project Context">
                        <MnemoGrid>
                          {((appState?.ragResponse as any)?.project_chunks ?? []).map((chunk: any, idx: number) => (
                            <MnemoChunkCard
                              key={`proj-${idx}`}
                              chunkIndex={idx + 1}
                              filePath={chunk.document_path || 'project'}
                              text={typeof chunk === 'string' ? chunk : chunk.text || `${chunk}`}
                              tags={chunk.tags || ['project']}
                              score={chunk.score}
                            />
                          ))}
                        </MnemoGrid>
                      </MnemoCollapse>
                      <MnemoCollapse title="Domain Context">
                        <MnemoGrid>
                          {((appState?.ragResponse as any)?.domain_chunks ?? []).map((chunk: any, idx: number) => (
                            <MnemoChunkCard
                              key={`dom-${idx}`}
                              chunkIndex={idx + 1}
                              filePath={chunk.document_path || 'domain'}
                              text={typeof chunk === 'string' ? chunk : chunk.text || `${chunk}`}
                              tags={chunk.tags || ['domain']}
                              score={chunk.score}
                            />
                          ))}
                        </MnemoGrid>
                      </MnemoCollapse>
                      <MnemoCollapse title="Company Context">
                        <MnemoGrid>
                          {((appState?.ragResponse as any)?.company_chunks ?? []).map((chunk: any, idx: number) => (
                            <MnemoChunkCard
                              key={`comp-${idx}`}
                              chunkIndex={idx + 1}
                              filePath={chunk.document_path || 'company'}
                              text={typeof chunk === 'string' ? chunk : chunk.text || `${chunk}`}
                              tags={chunk.tags || ['company']}
                              score={chunk.score}
                            />
                          ))}
                        </MnemoGrid>
                      </MnemoCollapse>
                    </div>
                  ) : (
                    <div className="mt-4 text-sm opacity-70">No chunks available yet. Run a query to see results.</div>
                  )}
                    <div
                      className={`transition-all ease-out duration-300 ${
                        appState?.ragResponse
                          ? 'translate-y-0 opacity-100'
                          : 'translate-y-5 opacity-0 pointer-events-none'
                      }`}
                    >
                      {appState?.loading ? (
                        <div className="mt-3 space-y-2">
                          <MnemoSkeleton />
                          <MnemoSkeleton />
                          <MnemoSkeleton />
                        </div>
                      ) : (
                        <pre
                          className="mt-2 overflow-auto rounded bg-gray-900/50 p-3 text-sm"
                          style={{ color: 'var(--mnemo-text)' }}
                        >
                          {JSON.stringify(appState?.ragResponse, null, 2)}
                        </pre>
                      )}
                    </div>
                  </div>
                  <MnemoCard title="Query Metadata">
                    <div className="space-y-2 text-sm">
                      <div>Vector hits: {metadata?.vector_hits ?? 0}</div>
                      <div>Graph expansion depth: {metadata?.graph_depth ?? 0}</div>
                      <div>Response time: {metadata?.response_time_ms ?? 0}ms</div>
                    </div>
                  </MnemoCard>
                </div>
                {showDebug && (
                  <div className="mt-4">
                    <MnemoCard title="RAG Debug Info">
                      <RagDebugPanel loading={debugLoading} candidates={debugResults} />
                    </MnemoCard>
                  </div>
                )}
              </>
            ) : (
              <div className="space-y-4">
                <div className="space-y-2">
                  <label className="block text-sm font-semibold">Ranking Profile</label>
                  <select
                    value={rankingProfile}
                    onChange={(e) => setRankingProfile(e.target.value)}
                    className="w-full rounded border p-3 bg-[var(--mnemo-bg-light)] text-[var(--mnemo-text)]"
                  >
                    <option value="default">Default</option>
                    <option value="semantic-heavy">Semantic Heavy</option>
                    <option value="graph-heavy">Graph Heavy</option>
                  </select>
                </div>
                <div className="space-y-2">
                  <label className="flex items-center gap-2 text-sm font-semibold">
                    <input
                      type="checkbox"
                      checked={expandGraph}
                      onChange={(e) => setExpandGraph(e.target.checked)}
                    />
                    Expand Graph Context
                  </label>
                  <label className="flex items-center gap-2 text-sm font-semibold">
                    <input type="checkbox" checked={showRaw} onChange={(e) => setShowRaw(e.target.checked)} />
                    Show Raw JSON
                  </label>
                </div>
              </div>
            )}
          </div>
        </MnemoCard>
      </section>
    </ContentContainer>
  )
}
