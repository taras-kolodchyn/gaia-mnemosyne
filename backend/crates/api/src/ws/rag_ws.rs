use axum::extract::ws::{Message, WebSocket};
use axum::{extract::WebSocketUpgrade, response::IntoResponse};

use crate::ws::hub::GLOBAL_WS_HUB;

pub async fn rag_ws(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
    let mut rx = GLOBAL_WS_HUB.subscribe();
    let _ = socket.send(Message::Text(r#"{"event":"connected"}"#.to_string())).await;

    while let Ok(msg) = rx.recv().await {
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(&msg) {
            if let Some(event) = value.get("event").and_then(|e| e.as_str()) {
                if event == "rag_processing" || event == "rag_done" {
                    let _ = socket.send(Message::Text(msg.clone())).await;
                }
            }
        }
    }
}
