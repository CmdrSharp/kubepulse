use crate::{router::router, state::AppState, ws::websocket, Args};
use axum::{
    extract::{ConnectInfo, State, WebSocketUpgrade},
    headers,
    response::IntoResponse,
    ServiceExt, TypedHeader,
};
use std::{net::SocketAddr, sync::Arc};
use tokio::signal;

/// HTTP Server
pub async fn serve(args: Args) {
    let addr = SocketAddr::from(([0, 0, 0, 0], args.port));
    let app = router(args.clone()).await;

    tracing::info!("Listening on {}{}", addr, args.basepath);

    axum::Server::bind(&addr)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .with_graceful_shutdown(hyper_shutdown())
        .await
        .expect("Failed to bind to address");
}

/// Handler for the inbound HTTP request. This is where we upgrade the connection
/// to a WebSocket connection.
pub async fn socket_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    user_agent: Option<TypedHeader<headers::UserAgent>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    let user_agent = if let Some(TypedHeader(user_agent)) = user_agent {
        user_agent.to_string()
    } else {
        "unknown".to_string()
    };

    // Increase number of clients
    state.add_client().await;

    tracing::info!(
        "New connection from {} ({}). Total connections: {}",
        addr,
        user_agent,
        state.clients().await,
    );

    // Upgrade connection to websocket
    ws.on_upgrade(move |socket| websocket(socket, state, addr))
}

// Graceful shutdown handler for Axum/Hyper
async fn hyper_shutdown() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install CTRL+C signal handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => tracing::info!("CTRL+C received"),
        _ = terminate => tracing::info!("SIGTERM received"),
    };

    tracing::info!("Shutting down");
}
