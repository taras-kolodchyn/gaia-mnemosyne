use axum::extract::ws::{Message, WebSocket};
use axum::{extract::WebSocketUpgrade, response::IntoResponse};
use futures_util::{SinkExt, StreamExt};

use crate::ws::hub::GLOBAL_WS_HUB;

pub async fn all_ws(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(socket: WebSocket) {
    let mut rx = GLOBAL_WS_HUB.subscribe();
    let (mut sender, _receiver) = socket.split();
    // optional: send connected event
    let _ = sender.send(Message::Text(r#"{"event":"connected"}"#.to_string())).await;

    while let Ok(msg) = rx.recv().await {
        let _ = sender.send(Message::Text(msg)).await;
    }
}
