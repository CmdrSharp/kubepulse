use clap::{command, Parser};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod info;
mod middleware;
mod router;
mod server;
mod state;
mod template;
mod ws;

/// KubePulse - Kubernetes cluster test application
#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// The port to listen on.
    #[arg(
        short,
        long,
        default_value_t = 80,
        value_parser = clap::value_parser!(u16).range(1..),
        env = "KUBEPULSE_PORT"
    )]
    port: u16,

    /// The base path to serve behind.
    #[arg(short, long, default_value = "/", env = "KUBEPULSE_BASEPATH")]
    basepath: String,

    /// Static assets directory
    #[arg(
        short,
        long,
        default_value = "/app/assets",
        env = "KUBEPULSE_ASSETS_DIR"
    )]
    assets_dir: String,
}

#[tokio::main]
async fn main() {
    // Parse CLI Args
    let args = Args::parse();

    // Set up tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "kubepulse=info,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    server::serve(args).await;
}
