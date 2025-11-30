export function MnemoStatusBadge({ status }: { status: 'UP' | 'DOWN' | 'DEGRADED' | string }) {
  const base = 'px-3 py-1 text-sm rounded-full font-semibold'
  if (status === 'UP') {
    return <span className={`${base} bg-green-900/40 text-green-300`}>UP</span>
  }
  if (status === 'DOWN') {
    return <span className={`${base} bg-red-900/40 text-red-300`}>DOWN</span>
  }
  return <span className={`${base} bg-yellow-900/40 text-yellow-300`}>DEGRADED</span>
}
