type TimelineStatus = 'pending' | 'running' | 'success' | 'failed' | 'almost_done'

const steps: TimelineStatus[] = ['pending', 'running', 'success']

export function JobTimeline({ status }: { status: TimelineStatus }) {
// Map statuses to a visual pattern of filled/empty nodes.
// pending: ○ ○ ○
// running: ● ○ ○ (pulse middle)
// success: ● ● ●
// failed:  ● ● ✖

  return (
    <div className="flex items-center gap-2">
      {steps.map((step, idx) => {
        const isPending = status === 'pending'
        const isRunning = status === 'running'
        const isAlmost = status === 'almost_done'
        const isSuccess = status === 'success'
        const isFailed = status === 'failed'

        let color = 'bg-gray-600'
        if (isPending) {
          color = 'bg-transparent border border-gray-600'
        } else if (isRunning) {
          color = idx === 0 ? 'bg-green-500' : idx === 1 ? 'bg-blue-400 animate-pulse' : 'bg-transparent border border-gray-600'
        } else if (isAlmost) {
          color = idx <= 1 ? 'bg-green-500' : 'bg-transparent border border-gray-600'
        } else if (isSuccess) {
          color = 'bg-green-500'
        } else if (isFailed) {
          color = idx === 2 ? 'bg-red-500' : 'bg-green-500'
        }

        const lineColor =
          isFailed && idx >= 1
            ? 'border-red-500'
            : isSuccess || isAlmost || isRunning
              ? 'border-green-500'
              : 'border-gray-700'

        return (
          <div key={step} className="flex items-center gap-2">
            <div
              className={`h-3 w-3 rounded-full transition-all ${color}`}
              title={step}
            />
            {idx < steps.length - 1 && (
              <div className={`w-10 border-t border-dashed ${lineColor} transition-all`} />
            )}
          </div>
        )
      })}
    </div>
  )
}
