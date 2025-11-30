import { MnemoStatusBadge } from './MnemoStatusBadge'

export function MnemoTable({ columns, rows }: { columns: string[]; rows: Record<string, string>[] }) {
  return (
    <table className="w-full overflow-hidden rounded-xl bg-[var(--mnemo-card)] text-[var(--mnemo-text)]">
      <thead>
        <tr className="bg-black/30">
          {columns.map((c) => (
            <th key={c} className="px-4 py-2 text-left">
              {c}
            </th>
          ))}
        </tr>
      </thead>
      <tbody>
        {rows.map((r, i) => (
          <tr key={i} className="border-t border-black/20">
            {columns.map((c) => (
              <td key={c} className="px-4 py-2 opacity-80">
                {c === 'status' && r[c] ? <MnemoStatusBadge status={r[c]} /> : r[c]}
              </td>
            ))}
          </tr>
        ))}
      </tbody>
    </table>
  )
}
