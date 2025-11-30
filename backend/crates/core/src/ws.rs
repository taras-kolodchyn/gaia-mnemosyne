use once_cell::sync::Lazy;
use tokio::sync::broadcast;

/// Shared WebSocket broadcast hub for Mnemosyne events.
pub struct WsHub {
    pub tx: broadcast::Sender<String>,
}

impl WsHub {
    pub fn new() -> Self {
        let (tx, _rx) = broadcast::channel(100);
        Self { tx }
    }

    pub fn broadcast(&self, msg: String) {
        tracing::info!("WS broadcast: {}", msg);
        let _ = self.tx.send(msg);
    }
}

/// Global hub instance for all WS endpoints.
pub static WS_HUB: Lazy<WsHub> = Lazy::new(WsHub::new);

/// Helper to broadcast without importing the hub instance everywhere.
pub fn broadcast(msg: String) {
    WS_HUB.broadcast(msg);
}
