import { createContext, useContext, useEffect, useState } from 'react'
import { registerToastListener } from '../utils/toastBus'

type AppState = Record<string, unknown>

type AppStateValue = {
  state: AppState
  setState: React.Dispatch<React.SetStateAction<AppState>>
  ragResponse: unknown
  setRagResponse: React.Dispatch<React.SetStateAction<unknown>>
  loading: boolean
  setLoading: React.Dispatch<React.SetStateAction<boolean>>
  ragHistory: { query: string; time: string }[]
  addRagHistoryEntry: (query: string) => void
  toasts: { id: number; message: string }[]
  addToast: (message: string) => void
  removeToast: (id: number) => void
}

const AppStateContext = createContext<AppStateValue | null>(null)

export function AppStateProvider({ children }: { children: React.ReactNode }) {
  const [state, setState] = useState<AppState>({})
  const [ragResponse, setRagResponse] = useState<unknown>(null)
  const [loading, setLoading] = useState(false)
  const [ragHistory, setRagHistory] = useState<{ query: string; time: string }[]>([])
  const [toasts, setToasts] = useState<{ id: number; message: string }[]>([])

  const addRagHistoryEntry = (query: string) => {
    const time = new Date().toLocaleTimeString()
    setRagHistory((prev) => [{ query, time }, ...prev].slice(0, 20))
  }

  const addToast = (message: string) => {
    const id = Date.now()
    setToasts((prev) => [...prev, { id, message }])
    setTimeout(() => removeToast(id), 5000)
  }

  const removeToast = (id: number) => {
    setToasts((prev) => prev.filter((t) => t.id !== id))
  }

  useEffect(() => {
    registerToastListener(addToast)
  }, [])

  return (
    <AppStateContext.Provider
      value={{
        state,
        setState,
        ragResponse,
        setRagResponse,
        loading,
        setLoading,
        ragHistory,
        addRagHistoryEntry,
        toasts,
        addToast,
        removeToast,
      }}
    >
      {children}
    </AppStateContext.Provider>
  )
}

export function useAppState() {
  return useContext(AppStateContext)
}
