import { useEffect, useState } from 'react'
import { ContentContainer } from '../components/ContentContainer'
import { MnemoSection } from '../components/MnemoSection'
import { PageTitle } from '../components/PageTitle'
import { MnemoCard } from '../components/MnemoCard'
import { MnemoModal } from '../components/MnemoModal'
import { MnemoStatusCard } from '../components/MnemoStatusCard'
import { mnemoGet } from '../api/client'
import { MnemoGrid } from '../components/MnemoGrid'

export function Sources() {
  const [search, setSearch] = useState('')
  const [animate, setAnimate] = useState(true)
  const [openModal, setOpenModal] = useState(false)
  const [providerType, setProviderType] = useState('filesystem')
  const [providerPath, setProviderPath] = useState('')
  const [enabled, setEnabled] = useState(true)
  const [providers, setProviders] = useState<{ name: string; status: 'enabled' | 'disabled' }[]>([])

  async function loadProviders() {
    try {
      const data = await mnemoGet('/v1/providers')
      const mapped: { name: string; status: 'enabled' | 'disabled' }[] = [
        { name: 'Filesystem', status: data.filesystem.enabled ? 'enabled' : 'disabled' },
        { name: 'GitHub', status: data.github.enabled ? 'enabled' : 'disabled' },
      ]
      setProviders(mapped)
    } catch {
      setProviders([])
    }
  }

  useEffect(() => {
    setAnimate(false)
    const id = requestAnimationFrame(() => setAnimate(true))
    return () => cancelAnimationFrame(id)
  }, [search])

  useEffect(() => {
    loadProviders()
  }, [])

  const filtered = providers.filter((s) => s.name.toLowerCase().includes(search.toLowerCase()))

  return (
    <ContentContainer>
      <section className="space-y-6 mb-10">
        <PageTitle title="Sources" subtitle="Manage data providers" />
        <MnemoSection title="Sources">
          <div className="mb-4 flex items-center gap-4">
            <input
              type="text"
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              placeholder="Search providers..."
              className="w-full rounded border px-3 py-2"
              style={{ background: 'var(--mnemo-bg-light)', color: 'var(--mnemo-text)' }}
            />
            <button
              className="rounded px-4 py-2 text-black"
              style={{ background: 'var(--mnemo-accent)' }}
              onClick={() => setOpenModal(true)}
            >
              Add Provider
            </button>
          </div>
          <div className="mb-2 text-sm font-semibold text-[var(--mnemo-text)] opacity-80">Available Providers</div>
          <div className="transition-all duration-300 ease-out">
            <MnemoGrid>
              {filtered.map((s, idx) => (
                <div
                  key={`${s.name}-${idx}-${search}`}
                  className="transition-all duration-200 hover:shadow-[0_0_12px_rgba(212,166,87,0.2)] hover:scale-[1.01]"
                >
                  <MnemoCard title={s.name}>
                    <div
                      className={`flex items-center justify-between transition-all duration-300 ease-out ${
                        animate ? 'opacity-100 translate-y-0' : 'opacity-0 translate-y-2'
                      }`}
                      style={{ transitionDelay: `${idx * 40}ms` }}
                    >
                      <span>{s.name}</span>
                      <MnemoStatusCard name={s.name} status={s.status === 'enabled' ? 'UP' : 'DOWN'} />
                    </div>
                  </MnemoCard>
                </div>
              ))}
              {filtered.length === 0 && (
                <div className="text-sm text-muted-foreground">No providers match your search.</div>
              )}
            </MnemoGrid>
          </div>
        </MnemoSection>

        <MnemoModal open={openModal} onClose={() => setOpenModal(false)} title="Add Provider">
          <div className="space-y-3">
            <div className="space-y-1">
              <label className="text-sm font-semibold">Provider Type</label>
              <select
                value={providerType}
                onChange={(e) => setProviderType(e.target.value)}
                className="w-full rounded border px-3 py-2"
                style={{ background: 'var(--mnemo-bg-light)', color: 'var(--mnemo-text)' }}
              >
                <option value="filesystem">Filesystem</option>
                <option value="github">GitHub</option>
                <option value="custom">Custom</option>
              </select>
            </div>
            <div className="space-y-1">
              <label className="text-sm font-semibold">Path / Repo URL</label>
              <input
                type="text"
                value={providerPath}
                onChange={(e) => setProviderPath(e.target.value)}
                className="w-full rounded border px-3 py-2"
                style={{ background: 'var(--mnemo-bg-light)', color: 'var(--mnemo-text)' }}
              />
            </div>
            <label className="flex items-center gap-2 text-sm font-semibold">
              <input type="checkbox" checked={enabled} onChange={(e) => setEnabled(e.target.checked)} />
              Enabled
            </label>
            <div className="flex justify-end">
              <button
                className="rounded px-4 py-2 text-black"
                style={{ background: 'var(--mnemo-accent)' }}
                onClick={() => setOpenModal(false)}
              >
                Save
              </button>
            </div>
          </div>
        </MnemoModal>
      </section>
    </ContentContainer>
  )
}
