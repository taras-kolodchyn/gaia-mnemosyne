import { mnemoGet } from './client'

type RawNode = {
  id: string
  data?: { label?: string }
  label?: string
  type?: string
  node_type?: string
}

type RawSnapshot = {
  nodes: RawNode[]
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

  const nodes = snapshot.nodes.map((n, idx) => {
    const label = n.data?.label ?? n.label ?? ''
    return {
      id: n.id,
      position: { x: idx * 200, y: 0 },
      data: { label },
      type: n.type ?? n.node_type,
    }
  })

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
