use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};

/// Represents the types of messages that can be sent via the `broadcast::Sender`.
#[derive(Debug)]
pub enum Message {
    /// Indicates that the server is currently running
    ServerRunning,
    /// Indicates that the server is terminating
    Terminate,
}

/// Manages the server shutdown state and provides a mechanism to notify listeners of shutdown
/// events
#[derive(Debug)]
pub struct Shutdown {
    /// Tracks whether the server is in the process of shutting down.
    is_shutdown: bool,
    /// A shared, thread-safe sender for broadcasting shutdown-related messages.
    shutdown_tx: Arc<Mutex<broadcast::Sender<Message>>>,
}

impl Shutdown {
    /// Creates a new `Shutdown` instance.
    ///
    /// # Arguments
    ///
    /// * `shutdown_tx` - A shared, thread-safe `broadcast::Sender` used to notify subscribers of shutdown events.
    ///
    /// # Returns
    ///
    /// A `Shutdown` struct initialized with the provided `shutdown_tx`.
    pub fn new(shutdown_tx: Arc<Mutex<broadcast::Sender<Message>>>) -> Shutdown {
        return Shutdown {
            is_shutdown: false,
            shutdown_tx,
        };
    }

    /// Checks if the server is currently in the process of shutting down.
    ///
    /// # Returns
    ///
    /// * `true` if the server is shutting down.
    /// * `false` otherwise.
    pub fn is_shutdown(&self) -> bool {
        return self.is_shutdown;
    }

    /// Initiates the server shutdown process.
    ///
    /// This method:
    /// 1. Sets the `is_shutdown` flag to `true`.
    /// 2. Sends a `Message::Terminate` to all subscribers via the `broadcast::Sender`.
    ///
    /// # Panics
    ///
    /// This function will panic if the `send` operation on the `broadcast::Sender` fails.
    pub async fn initiate_shutdown(&mut self) {
        self.is_shutdown = true;
        self.shutdown_tx
            .lock()
            .await
            .send(Message::Terminate)
            .unwrap();
    }
}

impl Clone for Message {
    fn clone(&self) -> Self {
        match self {
            Message::ServerRunning => Message::ServerRunning,
            Message::Terminate => Message::Terminate,
        }
    }
}

impl PartialEq for Message {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Message::ServerRunning => match other {
                Message::ServerRunning => true,
                _ => false,
            },
            Message::Terminate => match other {
                Message::Terminate => true,
                _ => false,
            },
        }
    }
}
