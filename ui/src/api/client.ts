import { emitToast } from '../utils/toastBus'

const BASE = import.meta.env.VITE_API_URL || 'http://localhost:7700'

export async function mnemoPost(path: string, body: any) {
  try {
    const res = await fetch(`${BASE}${path}`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      mode: 'cors',
      body: JSON.stringify(body),
    })
    if (!res.ok) throw new Error(`HTTP ${res.status}`)
    return await res.json()
  } catch (e) {
    emitToast('Backend unreachable')
    return null
  }
}

export async function mnemoGet(path: string) {
  try {
    const res = await fetch(`${BASE}${path}`, { mode: 'cors' })
    if (!res.ok) throw new Error(`HTTP ${res.status}`)
    return await res.json()
  } catch (e) {
    emitToast('Backend unreachable')
    return null
  }
}
