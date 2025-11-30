import { useEffect, useState } from 'react'
import { MnemoCard } from '../components/MnemoCard'
import { MnemoSection } from '../components/MnemoSection'
import { PageTitle } from '../components/PageTitle'
import { RAGActivityWidget } from '../components/RAGActivityWidget'
import { AlertsWidget } from '../components/AlertsWidget'
import { ContentContainer } from '../components/ContentContainer'
import { MnemoGrid } from '../components/MnemoGrid'
import { mnemoGet } from '../api/client'

export function Dashboard() {
  const [metrics, setMetrics] = useState<{ qdrant?: number; redis?: number; surreal?: number }>({})

  useEffect(() => {
    async function load() {
      try {
        const q = await mnemoGet('/v1/metrics?qdrant_latency')
        const r = await mnemoGet('/v1/metrics?redis_ping')
        const s = await mnemoGet('/v1/metrics?surreal_latency')
        setMetrics({
          qdrant: Number(q?.value ?? q ?? 0),
          redis: Number(r?.value ?? r ?? 0),
          surreal: Number(s?.value ?? s ?? 0),
        })
      } catch (e) {
        console.warn('metrics fetch failed', e)
      }
    }
    load()
  }, [])

  return (
    <ContentContainer>
      <section className="space-y-6 mb-8">
        <PageTitle title="Dashboard" subtitle="System overview for Gaia Mnemosyne" />

        <MnemoSection title="System Status">
          <MnemoGrid>
            <MnemoCard title="System Health">
              Live health endpoints, service uptime, and dependency status.
            </MnemoCard>
            <MnemoCard title="Dependencies">
              <div className="space-y-2 text-sm">
                <div>Qdrant latency: {metrics.qdrant ?? '-'} ms</div>
                <div>Redis ping: {metrics.redis ?? '-'} ms</div>
                <div>SurrealDB: {metrics.surreal ?? '-'} ms</div>
              </div>
            </MnemoCard>
            <MnemoCard title="Alerts">
              <AlertsWidget />
            </MnemoCard>
          </MnemoGrid>
        </MnemoSection>

        <MnemoSection title="Ingestion Overview">
          <MnemoGrid>
            <MnemoCard title="Ingestion Jobs">
              Monitor filesystem/GitHub scans and reindex tasks.
            </MnemoCard>
            <MnemoCard title="Providers">
              Overview of enabled providers and recent activity.
            </MnemoCard>
            <MnemoCard title="Queue Depth">
              Placeholder for queue metrics and throughput.
            </MnemoCard>
          </MnemoGrid>
        </MnemoSection>

        <MnemoSection title="RAG Activity">
          <MnemoGrid>
            <MnemoCard title="Queries">
              <RAGActivityWidget />
            </MnemoCard>
            <MnemoCard title="Embeddings">
              Placeholder for embedding latency and counts.
            </MnemoCard>
            <MnemoCard title="Graph">
              Placeholder for graph expansion metrics.
            </MnemoCard>
          </MnemoGrid>
        </MnemoSection>
      </section>
    </ContentContainer>
  )
}
