import { BrowserRouter, Route, Routes } from 'react-router-dom'
import { AppStateProvider } from './context'
import { Layout } from './components/Layout'
import { Dashboard } from './pages/Dashboard'
import { Sources } from './pages/Sources'
import { RAGPlayground } from './pages/RAGPlayground'
import { SystemStatus } from './pages/SystemStatus'
import { PageTransition } from './components/PageTransition'
import { PipelineMonitor } from './pages/PipelineMonitor'
import { GraphExplorer } from './pages/GraphExplorer'
import { WSProvider } from './context/WSContext'
import { MnemoToast } from './components/MnemoToast'
import { LogsDrawer } from './components/LogsDrawer'

function App() {
  return (
    <WSProvider>
      <AppStateProvider>
        <BrowserRouter>
          <Layout>
            <Routes>
              <Route path="/" element={<PageTransition><Dashboard /></PageTransition>} />
              <Route path="/sources" element={<PageTransition><Sources /></PageTransition>} />
              <Route path="/rag" element={<PageTransition><RAGPlayground /></PageTransition>} />
              <Route path="/status" element={<PageTransition><SystemStatus /></PageTransition>} />
              <Route path="/pipeline" element={<PageTransition><PipelineMonitor /></PageTransition>} />
              <Route path="/graph" element={<PageTransition><GraphExplorer /></PageTransition>} />
            </Routes>
          </Layout>
          <LogsDrawer />
          <MnemoToast />
        </BrowserRouter>
      </AppStateProvider>
    </WSProvider>
  )
}

export default App
