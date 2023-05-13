use crate::info::KubeInfo;
use crate::state::AppState;
use axum::extract::{ws::Message, ws::WebSocket};
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::{fmt::Debug, net::SocketAddr, sync::Arc};

#[derive(Deserialize, Serialize, Debug, Clone)]
struct TickInfo {
    time: String,
    pod: String,
    node: String,
    clients: u32,
}

/// Websocket handler / state machine
pub async fn websocket(socket: WebSocket, state: Arc<AppState>, who: SocketAddr) {
    let (mut sender, mut receiver) = socket.split();

    let send_state = Arc::clone(&state);

    // Send updates to the client every second
    let mut send_task = tokio::spawn(async move {
        loop {
            tracing::debug!("Sending update to client {}", who);

            let info = serde_json::to_string(&KubeInfo::new(&send_state).await).unwrap();

            if let Err(e) = sender.send(Message::Text(info)).await {
                tracing::error!("Failed to send message: {}", e);
            }

            tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
        }
    });

    // Receive messages from the client
    let mut recv_task = tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            tracing::debug!("Received message from {}: {:?}", who, msg);
        }
    });

    // If any tasks exit, abort the other
    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    };

    tracing::info!("Client {} disconnected", who);

    // Decrease number of clients
    state.drop_client().await;
}
