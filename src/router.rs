use crate::{
    middleware::normalize_path::{NormalizePath, NormalizePathLayer},
    server::socket_handler,
    state::AppState,
    template::{HtmlTemplate, IndexTemplate},
    Args,
};
use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    response::IntoResponse,
    routing::get,
    Router,
};
use clap::crate_version;
use std::{env::var_os, path::PathBuf, sync::Arc};
use tower::Layer;
use tower_http::{
    services::ServeDir,
    trace::{DefaultMakeSpan, TraceLayer},
};

/// Router setup
pub async fn router(args: Args) -> NormalizePath<Router> {
    let app_state = Arc::new(AppState::new(args.clone()).await);
    let assets_dir = PathBuf::from(args.assets_dir);

    NormalizePathLayer::trim_trailing_slash().layer(
        Router::new()
            .route(&get_route(&args.basepath, ""), get(index))
            .route(&get_route(&args.basepath, "/health"), get(alive))
            .nest_service(
                &get_route(&args.basepath, "/assets"),
                get(|request: Request<Body>| async {
                    let service = ServeDir::new(assets_dir);
                    tower::ServiceExt::oneshot(service, request).await
                }),
            )
            .route(&get_route(&args.basepath, "/ws"), get(socket_handler))
            .layer(
                TraceLayer::new_for_http()
                    .make_span_with(DefaultMakeSpan::default().include_headers(true)),
            )
            .with_state(app_state),
    )
}

/// Index handler
async fn index(State(app_state): State<Arc<AppState>>) -> impl IntoResponse {
    let namespace = var_os("KUBERNETES_NAMESPACE")
        .unwrap_or("-".into())
        .to_str()
        .unwrap()
        .to_string();

    let version = crate_version!().to_string();
    let asset_path = get_route(&app_state.args().basepath, "/assets")
        .trim_start_matches('/')
        .to_string();

    HtmlTemplate(IndexTemplate {
        namespace,
        version,
        asset_path,
    })
}

/// Aliveness handler
async fn alive() -> impl IntoResponse {
    (StatusCode::OK, "OK").into_response()
}

/// Get normalized route
fn get_route(basepath: &str, path: &str) -> String {
    // Normalize basepath
    let basepath = if basepath == "/" {
        basepath
    } else {
        basepath.strip_suffix('/').unwrap_or(basepath)
    };

    // Normalize path
    let path = if path == "/" {
        ""
    } else {
        path.strip_suffix('/').unwrap_or(path)
    };

    // Combine the paths and strip any double slashes
    format!("{}{}", basepath, path).replace("//", "/")
}
