export function MnemoStatusIcon({ status }: { status: 'UP' | 'DOWN' | 'UNKNOWN' | string }) {
  if (status === 'UP') {
    return <span className="text-green-400 animate-pulse">●</span>
  }
  if (status === 'DOWN') {
    return <span className="text-red-400">●</span>
  }
  return <span className="text-gray-400 animate-pulse">●</span>
}
