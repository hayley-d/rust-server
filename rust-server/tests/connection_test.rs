use std::sync::Arc;

use async_server::{Message, Shutdown};
use tokio::sync::{broadcast, Mutex};

#[tokio::test]
async fn test_shutdown() {
    let (tx, mut rx) = broadcast::channel(10);
    let tx = Arc::new(Mutex::new(tx));

    // Create new shutdown
    let mut shutdown = Shutdown::new(Arc::clone(&tx));
    // Initiate shutdown
    assert_eq!(shutdown.is_shutdown(), false);

    shutdown.initiate_shutdown().await;

    assert_eq!(rx.recv().await.unwrap(), Message::Terminate);
    assert_eq!(shutdown.is_shutdown(), true);
}
