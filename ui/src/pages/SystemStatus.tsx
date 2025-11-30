import { useEffect, useRef, useState } from 'react'
import { motion } from 'framer-motion'
import { MnemoSection } from '../components/MnemoSection'
import { PageTitle } from '../components/PageTitle'
import { MnemoCollapse } from '../components/MnemoCollapse'
import { ContentContainer } from '../components/ContentContainer'
import { MnemoStatusCard } from '../components/MnemoStatusCard'
import { SystemBanner } from '../components/SystemBanner'
import { MnemoGrid } from '../components/MnemoGrid'
import { mnemoGet } from '../api/client'
import { useWS } from '../context/WSContext'

export function SystemStatus() {
  const [apiUp, setApiUp] = useState<boolean | null>(null)
  const [qdrantUp, setQdrantUp] = useState<boolean | null>(null)
  const [surrealUp, setSurrealUp] = useState<boolean | null>(null)
  const [redisUp, setRedisUp] = useState<boolean | null>(null)
  const [postgresUp, setPostgresUp] = useState<boolean | null>(null)
  const { addLocalLog } = useWS()
  const prevAnyDown = useRef(false)

  async function fetchStatus() {
    try {
      const res = await mnemoGet('/v1/health')
      setApiUp(res.api === 'UP')
      setQdrantUp(res.qdrant === 'UP')
      setSurrealUp(res.surrealdb === 'UP')
      setRedisUp(res.redis === 'UP')
      setPostgresUp(res.postgres === 'UP')
    } catch {
      setApiUp(false)
      setQdrantUp(false)
      setSurrealUp(false)
      setRedisUp(false)
      setPostgresUp(false)
    }
  }

  useEffect(() => {
    fetchStatus()
    const id = setInterval(fetchStatus, 4000)
    return () => clearInterval(id)
  }, [])

  const apiStatus = apiUp === null ? 'UNKNOWN' : apiUp ? 'UP' : 'DOWN'
  const qdrantStatus = qdrantUp === null ? 'UNKNOWN' : qdrantUp ? 'UP' : 'DOWN'
  const surrealStatus = surrealUp === null ? 'UNKNOWN' : surrealUp ? 'UP' : 'DOWN'
  const redisStatus = redisUp === null ? 'UNKNOWN' : redisUp ? 'UP' : 'DOWN'
  const postgresStatus = postgresUp === null ? 'UNKNOWN' : postgresUp ? 'UP' : 'DOWN'
  const allUp = [apiStatus, qdrantStatus, surrealStatus, redisStatus, postgresStatus].every((s) => s === 'UP')
  const anyDown = [apiStatus, qdrantStatus, surrealStatus, redisStatus, postgresStatus].some((s) => s === 'DOWN')

  useEffect(() => {
    if (anyDown && !prevAnyDown.current) {
      addLocalLog(
        `Health degraded: api=${apiStatus}, qdrant=${qdrantStatus}, surreal=${surrealStatus}, redis=${redisStatus}, postgres=${postgresStatus}`,
      )
    }
    if (!anyDown && prevAnyDown.current) {
      addLocalLog('Health recovered: all services are UP')
    }
    prevAnyDown.current = anyDown
  }, [anyDown, apiStatus, qdrantStatus, surrealStatus, redisStatus, postgresStatus, addLocalLog])

  return (
    <ContentContainer>
      <section className="space-y-6 mb-10">
        <PageTitle title="System Status" />
        <SystemBanner allUp={allUp} anyDown={anyDown} />
        <MnemoSection title="System Status">
          <div className="space-y-4">
            <MnemoGrid>
              <motion.div initial={{ opacity: 0, y: 10 }} animate={{ opacity: 1, y: 0 }} transition={{ duration: 0.3 }}>
                <MnemoCollapse title="API Health">
                  <MnemoStatusCard name="API" status={apiStatus} />
                </MnemoCollapse>
              </motion.div>
              <motion.div initial={{ opacity: 0, y: 10 }} animate={{ opacity: 1, y: 0 }} transition={{ duration: 0.3 }}>
                <MnemoCollapse title="Dependencies">
                  <div className="space-y-3">
                    <MnemoStatusCard name="Qdrant" status={qdrantStatus} />
                    <MnemoStatusCard name="SurrealDB" status={surrealStatus} />
                    <MnemoStatusCard name="Redis" status={redisStatus} />
                    <MnemoStatusCard name="PostgreSQL" status={postgresStatus} />
                  </div>
                </MnemoCollapse>
              </motion.div>
            </MnemoGrid>
          </div>
        </MnemoSection>
      </section>
    </ContentContainer>
  )
}
