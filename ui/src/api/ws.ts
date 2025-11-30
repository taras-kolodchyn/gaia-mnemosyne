function broadcastStatus(connected: boolean, rtt?: number) {
  if (typeof window !== 'undefined' && typeof window.dispatchEvent === 'function') {
    window.dispatchEvent(new CustomEvent('mnemo-ws-status', { detail: { connected, rtt } }))
  }
}

export function createWebSocketClient(path: string, onMessage: (data: any) => void) {
  const url = `${import.meta.env.VITE_API_WS_URL || 'ws://localhost:7700'}${path}`
  const socket = new WebSocket(url)

  let pingInterval: ReturnType<typeof setInterval> | undefined
  let lastPing = 0

  socket.onopen = () => {
    console.log('[WS] connected:', url)
    broadcastStatus(true)
    pingInterval = setInterval(() => {
      try {
        lastPing = Date.now()
        socket.send(JSON.stringify({ event: 'ping', ts: lastPing }))
      } catch (e) {
        console.warn('WS ping send failed', e)
      }
    }, 10000)
  }
  socket.onerror = (e) => {
    console.error('[WS] error:', e)
    broadcastStatus(false)
  }
  socket.onmessage = (msg) => {
    try {
      const parsed = JSON.parse(msg.data)
      if (parsed?.event === 'pong' && parsed?.ts) {
        const rtt = Date.now() - parsed.ts
        broadcastStatus(true, rtt)
      }
      onMessage(parsed)
    } catch (_) {
      // ignore parse errors
    }
  }
  socket.onclose = () => {
    broadcastStatus(false)
    if (pingInterval) clearInterval(pingInterval)
  }

  return socket
}
