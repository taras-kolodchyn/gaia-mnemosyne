import dayjs from 'dayjs'
import relativeTime from 'dayjs/plugin/relativeTime'
dayjs.extend(relativeTime)
import { formatDate } from '../utils/date'

type Job = {
  id: string
  job_type: string
  status: string
  created_at: string
}

const statusColor = (status: string) => {
  const s = status.toLowerCase()
  if (s === 'success') return 'bg-green-500'
  if (s === 'running') return 'bg-blue-400 animate-pulse'
  if (s === 'failed') return 'bg-red-500'
  return 'bg-yellow-400'
}

export function JobsTimeline({ jobs }: { jobs: Job[] }) {
  if (!jobs || jobs.length === 0) {
    return <div className="text-sm opacity-70">No jobs executed yet.</div>
  }

  return (
    <div className="flex flex-col gap-3">
      {jobs.map((job) => (
        <div key={job.id} className="flex items-center gap-3">
          <div className={`w-3 h-3 rounded-full ${statusColor(job.status)}`} />
          <div className="flex flex-col">
            <span className="text-[var(--mnemo-text)] font-semibold text-sm">{job.job_type}</span>
            <span className="text-xs text-[var(--mnemo-text)]/70">
              {dayjs(job.created_at).fromNow()}
            </span>
          </div>
        </div>
      ))}
    </div>
  )
}
