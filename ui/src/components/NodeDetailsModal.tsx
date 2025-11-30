import { MnemoModal } from './MnemoModal'

export function NodeDetailsModal({
  open,
  onClose,
  node,
}: {
  open: boolean
  onClose: () => void
  node: {
    id: string
    type: string
    label: string
    neighbors: { id: string; label: string; node_type?: string; type?: string }[]
  } | null
}) {
  if (!node) return null

  const neighbors = node.neighbors || []

  return (
    <MnemoModal open={open} onClose={onClose} title="Node Details">
      <div className="space-y-2 text-sm">
        <div><strong>ID:</strong> {node.id}</div>
        <div><strong>Type:</strong> {node.type}</div>
        <div><strong>Label:</strong> {node.label}</div>
        <div><strong>Neighbors:</strong> {neighbors.length}</div>
        <div className="space-y-1">
          {neighbors.map((n) => (
            <div key={n.id} className="flex items-center justify-between">
              <span>{n.label}</span>
              <span className="opacity-70">{n.node_type || n.type || 'unknown'}</span>
            </div>
          ))}
        </div>
        <div><strong>Payload:</strong> (coming soon)</div>
      </div>
      <div className="mt-4 flex justify-end">
        <button
          className="rounded px-4 py-2 text-black"
          style={{ background: 'var(--mnemo-accent)' }}
          onClick={onClose}
        >
          Close
        </button>
      </div>
    </MnemoModal>
  )
}
