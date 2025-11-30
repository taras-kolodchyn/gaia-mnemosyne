import { useEffect, useState } from 'react'
import ReactFlow, { Background, Controls, ReactFlowProvider, useReactFlow } from 'reactflow'
import 'reactflow/dist/style.css'
import { ContentContainer } from '../components/ContentContainer'
import { motion } from 'framer-motion'
import { fetchGraphSnapshot, fetchNodeMetadata } from '../api/graph'
import type { GraphNodeMetadata } from '../api/graph'
import { MnemoLoader } from '../components/MnemoLoader'
import { NodeDetailsModal } from '../components/NodeDetailsModal'
import { MnemoGraphLegend } from '../components/MnemoGraphLegend'
import { MnemoSkeleton } from '../components/MnemoSkeleton'
import { useWS } from '../context/WSContext'
import { MnemoCard } from '../components/MnemoCard'

function GraphExplorerContent() {
  const reactFlow = useReactFlow()
  const [nodes, setNodes] = useState<any[]>([])
  const [edges, setEdges] = useState<any[]>([])
  const [allNodes, setAllNodes] = useState<any[]>([])
  const [allEdges, setAllEdges] = useState<any[]>([])
  const [selectedNode, setSelectedNode] = useState<{ id: string; data: { label: string } } | null>(null)
  const [loading, setLoading] = useState(false)
  const [filterText, setFilterText] = useState('')
  const [filterType, setFilterType] = useState<'All' | 'Repo' | 'File' | 'Chunk'>('All')
  const [nodeMeta, setNodeMeta] = useState<GraphNodeMetadata | null>(null)
  const [metaLoading, setMetaLoading] = useState(false)
  const [detailsOpen, setDetailsOpen] = useState(false)
  const [searchResults, setSearchResults] = useState<any[]>([])
  const [hoveredNodeId, setHoveredNodeId] = useState<string | null>(null)
  const { messages } = useWS()
  const nodeIcon = (t?: string) =>
    t === 'repo' ? '📦' : t === 'file' ? '📄' : t === 'chunk' ? '✂️' : '🧩'

  function autoLayout() {
    const typeWeight = (id: string, label: string) => {
      const lower = label.toLowerCase()
      if (id.startsWith('repo:') || lower.includes('repo')) return 0
      if (id.startsWith('file:') || lower.includes('file')) return 1
      if (id.startsWith('chunk:') || lower.includes('chunk')) return 2
      return 3
    };

    const laidOut = nodes.map((n: any, idx: number) => ({
      ...n,
      position: { x: idx * 200, y: typeWeight(n.id, n.data?.label ?? '') * 200 },
    }));
    setNodes(laidOut);
    setAllNodes(
      allNodes.map((n: any) => {
        const found = laidOut.find((m: any) => m.id === n.id);
        return found ? found : n;
      })
    );
  }
  const colorForType = (id: string) => {
    if (id.startsWith('repo') || id.includes('repo')) {
      return {
        background: '#1d4ed8',
        borderColor: '#bfdbfe',
        color: '#e5edff',
        borderRadius: 9999,
        padding: 6,
      }
    }
    if (id.startsWith('file') || id.includes('file')) {
      return {
        background: '#166534',
        borderColor: '#bbf7d0',
        color: '#e5ffef',
        borderRadius: 12,
        padding: 6,
      }
    }
    if (id.startsWith('chunk') || id.includes('chunk')) {
      return {
        background: '#f59e0b',
        borderColor: '#fef08a',
        color: '#1f1f1f',
        borderRadius: 16,
        padding: 6,
      }
    }
    return {}
  }

  async function loadGraph() {
    setLoading(true)
    const snapshot = await fetchGraphSnapshot()
    const colored = snapshot.nodes.map((n) => {
      const baseStyle = colorForType(n.id)
      const label = (n.data?.label ?? '').toString()
      const icon = n.id.startsWith('repo')
        ? '📦'
        : n.id.startsWith('file')
          ? '📄'
          : '✂️'
      return {
        ...n,
        data: { ...n.data, label: `${icon} ${label.slice(0, 40)}${label.length > 40 ? '…' : ''}` },
        style: baseStyle,
        className: 'mnemo-node',
      }
    })
    setAllNodes(colored)
    setAllEdges(snapshot.edges)
    setNodes(colored)
    setEdges(snapshot.edges)
    setLoading(false)
  }

  useEffect(() => {
    loadGraph()
  }, [])

  useEffect(() => {
    const last = messages[messages.length - 1]
    if (!last || last.event !== 'graph_update') return

    const data = last
    try {
      if (data.node) {
        const node = data.node
        setAllNodes((prev) => {
          if (prev.some((n) => n.id === node.id)) return prev
          const newNode = {
            id: node.id,
            data: { label: node.label },
            position: { x: prev.length * 80, y: prev.length * 40 },
            style: { ...colorForType(node.id), boxShadow: '0 0 12px rgba(212,166,87,0.35)' },
            className: 'animate-pulse',
          }
          const updated = [...prev, newNode]
          setNodes(updated)
          setTimeout(() => {
            setAllNodes((p) =>
              p.map((n) =>
                n.id === node.id
                  ? { ...n, className: undefined, style: { ...(n.style as any), boxShadow: undefined } }
                  : n,
              ),
            )
            setNodes((p) =>
              p.map((n) =>
                n.id === node.id
                  ? { ...n, className: undefined, style: { ...(n.style as any), boxShadow: undefined } }
                  : n,
              ),
            )
          }, 2000)
          return updated
        })
      }
      if (data.edge) {
        setAllEdges((prev) => {
          const exists = prev.some((e) => e.source === data.edge.source && e.target === data.edge.target)
          if (exists) return prev
          const newEdge = {
            id: data.edge.id || `${data.edge.source}-${data.edge.target}-${Date.now()}`,
            source: data.edge.source,
            target: data.edge.target,
          }
          const updated = [...prev, newEdge]
          setEdges(updated)
          return updated
        })
      }
    } catch (e) {
      console.warn('graph ws message parse failed', e)
    }
  }, [messages])

  useEffect(() => {
    const text = filterText.toLowerCase()
    const filteredNodes = allNodes.filter((n) => {
      const label = (n.data?.label ?? '').toLowerCase()
      const matchesText = !text || label.includes(text)
      const matchesType =
        filterType === 'All' ||
        (filterType === 'Repo' && label.includes('repo')) ||
        (filterType === 'File' && label.includes('file')) ||
        (filterType === 'Chunk' && label.includes('chunk'))
      return matchesText && matchesType
    })
    const filteredNodeIds = new Set(filteredNodes.map((n) => n.id))
    const filteredEdges = allEdges.filter((e) => filteredNodeIds.has(e.source) && filteredNodeIds.has(e.target))
    setNodes(filteredNodes)
    setEdges(filteredEdges)
    setSearchResults(
      allNodes
        .filter((n) => {
          const label = (n.data?.label ?? '').toLowerCase()
          return text && label.includes(text)
        })
        .slice(0, 10)
    )
  }, [filterText, filterType, allNodes, allEdges])

  return (
    <ContentContainer>
      <div className="text-[var(--mnemo-text)] mt-6 space-y-3 mb-10">
        <h1 className="mnemo-h1">Graph Explorer</h1>
        <p className="opacity-70">Explore document relationships, concepts and providers.</p>
        <div className="flex items-center justify-between">
          <div className="text-sm opacity-70">Snapshot of current graph state.</div>
          <div className="flex items-center gap-3">
            <input
              type="text"
              placeholder="Search nodes..."
              value={filterText}
              onChange={(e) => setFilterText(e.target.value)}
              className="rounded border border-gray-700 bg-[var(--mnemo-bg-2)] px-3 py-2 text-sm"
              style={{ color: 'var(--mnemo-text)' }}
            />
            <select
              value={filterType}
              onChange={(e) => setFilterType(e.target.value as any)}
              className="rounded border border-gray-700 bg-[var(--mnemo-bg-2)] px-3 py-2 text-sm"
              style={{ color: 'var(--mnemo-text)' }}
            >
              <option value="All">All</option>
              <option value="Repo">Repo</option>
              <option value="File">File</option>
              <option value="Chunk">Chunk</option>
            </select>
            <button
              className="rounded px-4 py-2 text-sm border border-[rgba(212,166,87,0.3)] bg-[var(--mnemo-bg-2)] text-[var(--mnemo-text)] mnemo-shadow-1 hover:mnemo-glow-gold"
              onClick={loadGraph}
              disabled={loading}
            >
              {loading ? <MnemoLoader /> : 'Refresh Graph'}
            </button>
            <button
              className="rounded px-4 py-2 text-sm border border-[rgba(212,166,87,0.3)] bg-[var(--mnemo-bg-2)] text-[var(--mnemo-text)] mnemo-shadow-1 hover:mnemo-glow-gold"
              onClick={autoLayout}
            >
              Auto Layout
            </button>
          </div>
        </div>
          <div className="grid gap-6 lg:grid-cols-[70%_30%] grid-cols-1 items-start">
          <div className="relative w-full h-[calc(100vh-160px)] rounded border border-gray-700 bg-[var(--mnemo-bg-light)] overflow-hidden">
            {nodes.length === 0 ? (
              <div className="flex h-full items-center justify-center text-sm opacity-70 text-[var(--mnemo-text)]">
                <div className="space-y-2 w-full max-w-md px-6">
                  <MnemoSkeleton />
                  <MnemoSkeleton />
                  <MnemoSkeleton />
                  <div className="text-center text-sm opacity-70 mt-2">Graph is empty — run ingestion</div>
                </div>
              </div>
            ) : (
              <ReactFlow
                nodes={nodes.map((n) => {
                  const isSelected = selectedNode && n.id === selectedNode.id
                  const baseHovered = hoveredNodeId || selectedNode?.id || null
                  if (!baseHovered) return n
                  const isAnchor = n.id === baseHovered
                  const isNeighbor =
                    edges.some((e) => e.source === baseHovered && e.target === n.id) ||
                    edges.some((e) => e.target === baseHovered && e.source === n.id)
                  const opacity = isAnchor ? 1 : isNeighbor ? 0.8 : 0.4
                  const glow = isAnchor ? '0 0 14px rgba(212,166,87,0.5)' : isNeighbor ? '0 0 10px rgba(147, 197, 253, 0.35)' : undefined
                  const scale = isAnchor ? 1.06 : 1
                  return {
                    ...n,
                    style: { ...(n.style as any), opacity, boxShadow: glow, transform: `scale(${scale})` },
                    className: isAnchor ? 'transition-transform duration-200' : n.className,
                  }
                })}
                edges={edges}
                fitView
                panOnScroll
                zoomOnScroll
                panOnDrag
                className="w-full h-full"
                onNodeClick={async (_, node) => {
                  setSelectedNode(node as any)
                  setHoveredNodeId(node.id)
                  setMetaLoading(true)
                  try {
                    const meta = await fetchNodeMetadata(node.id)
                    setNodeMeta(meta)
                  } catch (e) {
                    console.error('Failed to load node metadata', e)
                    setNodeMeta(null)
                  } finally {
                    setMetaLoading(false)
                  }
                }}
                onNodeMouseEnter={(_, node) => setHoveredNodeId(node.id)}
                onNodeMouseLeave={() => setHoveredNodeId(selectedNode?.id ?? null)}
              >
                <Background />
                <Controls />
              </ReactFlow>
            )}
              {filterText && searchResults.length > 0 && (
                <div className="absolute left-3 top-3 z-10 w-64 rounded border border-gray-700 bg-[var(--mnemo-bg-2)] p-2 text-xs shadow">
                  <div className="mb-1 text-[var(--mnemo-text)]">Results</div>
                  <div className="space-y-1 max-h-40 overflow-auto">
                    {searchResults.map((n) => (
                      <button
                        key={n.id}
                        className="w-full text-left rounded px-2 py-1 hover:bg-[var(--mnemo-bg-light)]"
                        onClick={() => {
                          reactFlow.fitView({ nodes: [{ id: n.id }], padding: 0.2 })
                          setSelectedNode(n)
                        }}
                      >
                        <div className="font-semibold text-[var(--mnemo-text)]">{n.data?.label ?? n.id}</div>
                        <div className="opacity-60">{n.id}</div>
                      </button>
                    ))}
                  </div>
                </div>
              )}
              <div className="absolute right-3 top-3 flex flex-col gap-2">
                <button
                  className="rounded px-3 py-2 text-sm border border-[rgba(212,166,87,0.3)] bg-[var(--mnemo-bg-2)] text-[var(--mnemo-text)] mnemo-shadow-1 hover:mnemo-glow-gold"
                  onClick={() => reactFlow.fitView()}
                >
                  Fit to Screen
                </button>
                <button
                  className="rounded px-3 py-2 text-sm border border-[rgba(212,166,87,0.3)] bg-[var(--mnemo-bg-2)] text-[var(--mnemo-text)] mnemo-shadow-1 hover:mnemo-glow-gold"
                  onClick={() => reactFlow.zoomTo(1)}
                >
                  Reset View
                </button>
              </div>
            </div>
            <div className="max-h-[calc(100vh-160px)] overflow-auto">
            <MnemoCard title="Node Inspector">
              {selectedNode ? (
                <motion.div
                  className="space-y-3"
                  initial={{ opacity: 0, y: 5 }}
                  animate={{ opacity: 1, y: 0 }}
                  transition={{ duration: 0.2 }}
                >
                  <div className="flex items-center gap-2 text-lg font-semibold">
                    <span className="text-xl">
                      {nodeIcon(nodeMeta?.node_type ?? 'unknown')}
                    </span>
                    <span className="truncate">{nodeMeta?.label ?? selectedNode.data.label}</span>
                  </div>
                  <div className="text-xs opacity-70 break-all">{selectedNode.id}</div>
                  {metaLoading ? (
                    <div className="space-y-1">
                      <MnemoSkeleton />
                      <MnemoSkeleton />
                    </div>
                  ) : (
                    <div className="bg-[var(--mnemo-bg-light)]/60 border border-[rgba(212,166,87,0.25)] rounded p-3 text-sm space-y-1">
                      <div className="flex justify-between">
                        <span className="opacity-70">Type</span>
                        <span className="font-semibold capitalize">{nodeMeta?.node_type ?? 'unknown'}</span>
                      </div>
                      <div className="flex justify-between">
                        <span className="opacity-70">ID</span>
                        <span className="font-semibold break-all text-right">{selectedNode.id}</span>
                      </div>
                      <div className="flex justify-between">
                        <span className="opacity-70">Neighbors</span>
                        <span className="font-semibold">{nodeMeta?.neighbors?.length ?? 0}</span>
                      </div>
                    </div>
                  )}

                  <div className="flex justify-end">
                    <button
                      className="mt-2 rounded px-3 py-2 text-sm border border-[rgba(212,166,87,0.4)] bg-[var(--mnemo-bg-2)] text-[var(--mnemo-text)] mnemo-shadow-1 hover:mnemo-glow-gold"
                      onClick={() => setDetailsOpen(true)}
                    >
                      View relationships
                    </button>
                  </div>
                </motion.div>
              ) : (
                <div>Select a node to see relationships and metadata.</div>
              )}
            </MnemoCard>
            <div className="mt-3"><MnemoGraphLegend /></div>
            <NodeDetailsModal
              open={detailsOpen}
              onClose={() => setDetailsOpen(false)}
              node={
                nodeMeta
                  ? {
                      id: nodeMeta.id,
                      type: nodeMeta.node_type,
                      label: nodeMeta.label,
                      neighbors: nodeMeta.neighbors,
                    }
                  : null
              }
            />
            </div>
        </div>
      </div>
    </ContentContainer>
  )
}

export function GraphExplorer() {
  return (
    <ReactFlowProvider>
      <GraphExplorerContent />
    </ReactFlowProvider>
  )
}
