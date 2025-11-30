import { createBrowserRouter } from 'react-router-dom'
import { Dashboard } from './pages/Dashboard'
import { RAGPlayground } from './pages/RAGPlayground'
import { Sources } from './pages/Sources'
import { SystemStatus } from './pages/SystemStatus'
import { PipelineMonitor } from './pages/PipelineMonitor'
import { GraphExplorer } from './pages/GraphExplorer'

export const router = createBrowserRouter([
  { path: '/', element: <Dashboard /> },
  { path: '/rag', element: <RAGPlayground /> },
  { path: '/sources', element: <Sources /> },
  { path: '/status', element: <SystemStatus /> },
  { path: '/pipeline', element: <PipelineMonitor /> },
  { path: '/graph', element: <GraphExplorer /> },
])
