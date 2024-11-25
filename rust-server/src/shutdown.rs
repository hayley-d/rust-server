use tokio::sync::broadcast;

/// Listens for the server shutdown signal.
///

#[derive(Debug)]
pub struct Shutdown {
    /// true is the shutdown signal has been recieved
    is_shutdown: bool,
    notify: broadcast::Receiver<()>,
}

impl Shutdown {
    /// creates a new shutdown struct with the provided broadcast::Reciever
    pub fn new(notify: broadcast::Reciever<()>) -> Shutdown {
        return Shutdown {
            is_shutdown: false,
            notify,
        };
    }

    /// Returns true if the server has shutdown otherwise it will return false.
    pub fn is_shudown(&self) -> bool {
        return self.is_shutdown;
    }

    /// Receive the shutdown signal, wait if need be.
    pub async fn receive_notice(&mut self) {
        if !self.is_shutdown {
            let _ = self.notify.recv().await();
            self.is_shutdown = true;
        }
    }
}
