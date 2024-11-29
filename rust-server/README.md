
# Asynchronous Multithreaded Rust Server

This project is an asynchronous TCP server written in Rust using the Tokio runtime. The server is designed to handle multiple client connections concurrently, supports graceful shutdown, and logs errors to a file for easier debugging and monitoring.

## Features
- **Asynchronous Connection Handling**: Efficiently handles multiple client connections concurrently using `tokio::spawn`.
- **Graceful Shutdown**: Supports stopping the server cleanly with `CTRL+C` using signal handling.
- **Error Logging**: Logs errors to a `server.log` file for persistent monitoring and debugging.
- **Dynamic Port Configuration**: Accepts a custom port via command-line arguments or defaults to port `7878`.



## How to Run
1. Clone the repository.
2. Ensure you have Rust installed. If not, install it via [Rustup](https://rustup.rs/).
3. Build the project using:
   ```bash
    cargo build --release
   ```
 4. Run the server:
 ```bash
./target/release/async_server [port]
```
Replace [port] with the desired port number. If no port is provided, the server defaults to 7878
## Optimizations

- **Error Handling**: The server uses a custom ErrorType enum to categorize and handle errors such as ConnectionFailed, Timeout, and more.
- **Graceful Shutdown**: Implements shutdown handling using tokio::signal::ctrl_c and a broadcast::channel to notify active connections to terminate.
- **Logging**: A Logger struct is used to persist errors in a server.log file, ensuring issues are traceable even after the server stops.


## Acknowledgements
This project would not have been possible without the help of the following resources, which provided invaluable insights into building an asynchronous TCP server in Rust:

 - [Tokio mini-redis](https://github.com/tokio-rs/mini-redis): A great example of an asynchronous Redis implementation, showcasing practical usage of the Tokio framework.
 - [Rust Book: Building a Web Server](https://doc.rust-lang.org/book/ch20-00-final-project-a-web-server.html): A step-by-step guide that lays the foundation for understanding networking concepts in Rust.

 - [Simple TCP Server in Rust](https://medium.com/go-rust/rust-day-7-tokio-simple-tcp-server-32c40f12e79b): A concise tutorial illustrating the creation of a TCP server using Rust and Tokio.



## Running Tests

To verify the functionality of the server, a suite of tests has been implemented following Rust's standard testing conventions. The test files are located in the tests directory, and you can execute them using the cargo test command:

```bash
  cargo test
```

**Notes on Testing:**
- The test suite includes integration tests to ensure the core functionality of the server, including connection handling, error responses, and graceful shutdown.

- Make sure no other processes are running on the default port (7878) during the test execution.

- Logging during tests will output to the same server.log file, which can be helpful for debugging.
## Authors

- [@hayley-d](https://www.github.com/hayley-d)


