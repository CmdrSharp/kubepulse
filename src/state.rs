use crate::Args;
use std::sync::Mutex;

pub struct AppState {
    clients: Mutex<i64>,
    args: Args,
}

impl AppState {
    /// Constructs a new AppState
    pub async fn new(args: Args) -> Self {
        Self {
            clients: Mutex::new(0),
            args,
        }
    }

    /// Gets current number of clients
    pub async fn clients(&self) -> i64 {
        *self.clients.lock().expect("Failed to lock mutex")
    }

    /// Adds a client to the state
    pub async fn add_client(&self) {
        let mut clients = self.clients.lock().expect("Failed to lock mutex");
        *clients += 1;
    }

    /// Removes a client from the state
    pub async fn drop_client(&self) {
        let mut clients = self.clients.lock().expect("Failed to lock mutex");
        *clients -= 1;
    }

    /// Getter for args
    pub fn args(&self) -> &Args {
        &self.args
    }
}
