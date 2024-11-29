use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};

#[derive(Debug)]
pub enum Message {
    ServerRunning,
    Terminate,
}

#[derive(Debug)]
pub struct Shutdown {
    is_shutdown: bool,
    shutdown_tx: Arc<Mutex<broadcast::Sender<Message>>>,
}

impl Shutdown {
    pub fn new(shutdown_tx: Arc<Mutex<broadcast::Sender<Message>>>) -> Shutdown {
        return Shutdown {
            is_shutdown: false,
            shutdown_tx,
        };
    }

    pub fn is_shutdown(&self) -> bool {
        return self.is_shutdown;
    }

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
