import { MnemoStatusIcon } from './MnemoStatusIcon'
import { MnemoStatusBadge } from './MnemoStatusBadge'
import { MnemoTooltip } from './MnemoTooltip'

export function MnemoStatusCard({ name, status }: { name: string; status: 'UP' | 'DOWN' | 'DEGRADED' | string }) {
  const glow = status === 'UP' ? 'shadow-[0_0_10px_rgba(212,166,87,0.25)]' : ''

  const description =
    name === 'Qdrant'
      ? 'Vector database'
      : name === 'SurrealDB'
        ? 'Graph + document store'
        : name === 'Redis'
          ? 'Cache + ephemeral storage'
          : name === 'API'
            ? 'Gaia Mnemosyne backend service'
            : ''

  return (
    <div
      className={`flex items-center justify-between rounded-lg border border-gray-700 bg-[var(--mnemo-bg-light)] px-4 py-3 shadow transition-all duration-200 hover:shadow-[0_0_12px_rgba(212,166,87,0.2)] hover:scale-[1.01] ${glow}`}
    >
      <div className="flex items-center gap-3">
        <MnemoStatusIcon status={status} />
        <div>
          <div className="text-sm opacity-70">Service</div>
          <MnemoTooltip text={description || name}>
            <div className="text-lg font-semibold">{name}</div>
          </MnemoTooltip>
        </div>
      </div>
      <MnemoStatusBadge status={status as any} />
    </div>
  )
}
