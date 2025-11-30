type JobState = 'pending' | 'running' | 'success' | 'failed'

const bgForStatus: Record<JobState, string> = {
  pending: 'bg-yellow-500/20 text-yellow-200',
  running: 'bg-blue-500/20 text-blue-200',
  success: 'bg-green-500/20 text-green-200',
  failed: 'bg-red-500/20 text-red-200',
}

export function JobStatusBadge({ status }: { status: JobState }) {
  const bg = bgForStatus[status] || 'bg-yellow-500/20 text-yellow-200'
  const label =
    status === 'success'
      ? 'SUCCESS'
      : status === 'failed'
        ? 'FAILED'
        : status === 'running'
          ? 'RUNNING'
          : 'PENDING'

  return (
    <span className={`inline-flex items-center rounded-full px-3 py-1 text-xs font-semibold ${bg}`}>
      {label}
    </span>
  )
}
