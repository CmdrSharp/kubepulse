use crate::state::AppState;
use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct KubeInfo {
    time: String,
    node: String,
    pod: String,
    message: String,
    clients: i64,
}

impl KubeInfo {
    /// Construct a new KubeInfo
    pub async fn new(state: &AppState) -> Self {
        Self {
            time: Self::get_time(),
            node: Self::get_env_var("KUBERNETES_NODE_NAME", None),
            pod: Self::get_env_var("KUBERNETES_POD_NAME", None),
            message: Self::get_env_var("KUBEPULSE_MESSAGE", Some("Hello Kubernetes!".to_string())),
            clients: state.clients().await,
        }
    }

    /// Get time in human readable format
    fn get_time() -> String {
        Utc::now().format("%Y-%m-%d %H:%M:%S").to_string()
    }

    /// Get variable from environment
    fn get_env_var(var: &str, default: Option<String>) -> String {
        let default = default.unwrap_or_else(|| "-".to_string());

        std::env::var_os(var)
            .unwrap_or(default.into())
            .to_str()
            .unwrap()
            .to_string()
    }
}
