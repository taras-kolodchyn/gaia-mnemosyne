type JobKind = 'filesystem_scan' | 'github_scan' | 'openapi_scan' | string

const colorForKind: Record<string, string> = {
  filesystem_scan: 'bg-blue-500/20 text-blue-200 border-blue-500/40',
  github_scan: 'bg-purple-500/20 text-purple-200 border-purple-500/40',
  openapi_scan: 'bg-orange-500/20 text-orange-200 border-orange-500/40',
}

export function JobTypeBadge({ kind }: { kind: JobKind }) {
  const color = colorForKind[kind] || 'bg-gray-500/20 text-gray-200 border-gray-500/40'
  return (
    <span className={`inline-flex items-center rounded-full px-2 py-0.5 text-xs font-semibold border ${color}`}>
      {kind}
    </span>
  )
}
