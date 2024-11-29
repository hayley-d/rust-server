use rust_server::connection::{connections::*, my_socket::*};
use rust_server::error::my_errors::*;
use rust_server::shutdown::*;
use std::env;
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex, Semaphore};

const DEFAULT_PORT: u16 = 7878;

#[tokio::main]
async fn main() -> Result<(), ErrorType> {
    let logger: Logger = Logger::new("server.log");

    let port: u16 = match env::args()
        .nth(1)
        .unwrap_or_else(|| DEFAULT_PORT.to_string())
        .parse()
    {
        Ok(p) => p,
        Err(_) => {
            let error = ErrorType::SocketError(String::from("Problem parsing port"));
            logger.log_error(&error);
            DEFAULT_PORT
        }
    };

    let socket = match create_socket(port) {
        Ok(s) => s,
        Err(e) => {
            logger.log_error(&e);
            panic!("Error creating socket, refer to the server log");
        }
    };

    let listener = match get_listener(socket) {
        Ok(s) => s,
        Err(e) => {
            logger.log_error(&e);
            panic!("Error creating listener, refer to the server log");
        }
    };

    let (tx, _rx) = broadcast::channel(10);
    let tx = Arc::new(Mutex::new(tx));

    let mut shutdown = Shutdown::new(Arc::clone(&tx));

    // Graceful shutdown using signal handling
    let shutdown_signal = tokio::signal::ctrl_c();

    let listener: Listener = Listener {
        listener,
        connection_limit: Arc::new(Semaphore::new(5)),
        shutdown_tx: Arc::clone(&tx),
    };

    tokio::select! {
        _ = run_server(listener,logger) => {
            println!("Server has stopped.");
        }
        _ = shutdown_signal => {
            println!("Shutdown signal received. Stopping server...");
            shutdown.initiate_shutdown().await;
        }
    }

    Ok(())
}

async fn run_server(mut listener: Listener, logger: Logger) -> Result<(), ErrorType> {
    let logger = Arc::new(Mutex::new(logger));
    listener.run(Arc::clone(&logger)).await?;
    return Ok(());
}
