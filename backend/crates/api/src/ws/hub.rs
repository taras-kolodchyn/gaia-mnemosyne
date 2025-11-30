use once_cell::sync::Lazy;
use tokio::sync::broadcast;

/// Global hub for all WebSocket events, backed by the core WS hub sender.
pub static GLOBAL_WS_HUB: Lazy<GlobalWsHub> = Lazy::new(GlobalWsHub::new);

pub struct GlobalWsHub {
    pub sender: broadcast::Sender<String>,
}

impl GlobalWsHub {
    pub fn new() -> Self {
        // Reuse the core hub sender so all existing broadcasts flow through.
        let sender = mnemo_core::ws::WS_HUB.tx.clone();
        Self { sender }
    }

    pub fn broadcast(&self, msg: impl Into<String>) {
        let _ = self.sender.send(msg.into());
    }

    pub fn subscribe(&self) -> broadcast::Receiver<String> {
        self.sender.subscribe()
    }
}
