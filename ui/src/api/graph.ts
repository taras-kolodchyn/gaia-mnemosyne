import { mnemoGet } from './client'

type RawSnapshot = {
  nodes: { id: string; label: string; type?: string }[]
  edges: { source: string; target: string; id?: string }[]
}

export type GraphNodeMetadata = {
  id: string
  label: string
  node_type: string
  edges_count: number
  neighbors: { id: string; node_type: string; label: string }[]
  extra: any
}

export async function fetchGraphSnapshot() {
  const res = await mnemoGet('/v1/graph/snapshot')
  const snapshot = res as RawSnapshot

  const nodes = snapshot.nodes.map((n, idx) => ({
    id: n.id,
    position: { x: idx * 200, y: 0 },
    data: { label: n.label },
  }))

  const edges = snapshot.edges.map((e, idx) => ({
    id: e.id ?? `edge-${idx}`,
    source: e.source,
    target: e.target,
  }))

  return { nodes, edges }
}

export async function fetchNodeMetadata(id: string): Promise<GraphNodeMetadata> {
  const res = await mnemoGet(`/v1/graph/node/${id}`)
  return res as GraphNodeMetadata
}
