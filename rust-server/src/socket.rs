/// This module provides utility functions for creating and managing sockets.
/// It utilizes the `socket2` crate for advanced socket operations and integrates with
/// Tokio's asynchronous networking capabilities.
pub mod my_socket {
    use crate::error::my_errors::ErrorType;
    use socket2::{Domain, Protocol, SockAddr, Socket, Type};
    use std::net::{Ipv6Addr, SocketAddrV6};
    use tokio::net::TcpListener;

    /// Creates an IPv6 TCP socket, binds it to the specified port, and prepares it to listen for incoming connections.
    ///
    /// # Arguments
    /// - `port`: The port number to bind the socket to.
    ///
    /// # Returns
    /// - `Ok(Socket)`: The created and configured socket.
    /// - `Err(ErrorType)`: An error if socket creation, binding, or listening fails.
    ///
    /// # Errors
    /// - `SocketError`: If creating, configuring, binding, or listening on the socket fails.
    ///
    /// # Example
    /// ```rust
    /// use rust_server::my_socket;
    /// let socket = my_socket::create_socket(8080).unwrap();
    /// ```
    pub fn create_socket(port: u16) -> Result<Socket, ErrorType> {
        // Create a new IPv6 TCP socket
        let socket = match Socket::new(Domain::IPV6, Type::STREAM, Some(Protocol::TCP)) {
            Ok(s) => s,
            Err(_) => {
                let error = ErrorType::SocketError(String::from("Creating socket"));
                return Err(error);
            }
        };

        // Enable address reuse to avoid "address already in use" errors
        match socket.set_reuse_address(true) {
            Ok(_) => (),
            Err(_) => {
                let error = ErrorType::SocketError(String::from(
                    "Problem when attempting to set reuse address",
                ));
                return Err(error);
            }
        };

        // Define the socket address as IPv6 loopback with specified port.
        let socket_address = SocketAddrV6::new(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1), port, 0, 0);
        let socket_address = SockAddr::from(socket_address);

        // Bind the socket to the address and port
        match socket.bind(&socket_address) {
            Ok(_) => (),
            Err(_) => {
                let error =
                    ErrorType::SocketError(String::from("Problem when binding address to socket"));
                return Err(error);
            }
        };

        // Start listening for incoming connections on the socket
        match socket.listen(128) {
            Ok(_) => (),
            Err(_) => {
                let error =
                    ErrorType::SocketError(String::from("Problem when binding address to socket"));
                return Err(error);
            }
        };

        println!("Listening on [::1]:{port}...");

        return Ok(socket);
    }

    /// Converts a socket into a Tokio `TcpListener` for asynchronous operations.
    ///
    /// # Arguments
    /// - `socket`: A pre-configured socket to be converted into a `TcpListener`.
    ///
    /// # Returns
    /// - `Ok(TcpListener)`: The converted asynchronous TCP listener.
    /// - `Err(ErrorType)`: An error if conversion or non-blocking setup fails.
    ///
    /// # Errors
    /// - `SocketError`: If setting the listener as non-blocking or conversion to a `TcpListener` fails.
    ///
    /// # Example
    /// ```rust
    /// use rust_server::my_socket;
    /// let socket = my_socket::create_socket(8080).unwrap();
    /// let listener = my_socket::get_listener(socket).unwrap();
    /// ```
    pub fn get_listener(socket: Socket) -> Result<TcpListener, ErrorType> {
        // Convert the socket2::Socket into a standard std::net::TcpListener
        let std_listener: std::net::TcpListener = socket.into();

        // Set the listener to non-blocking mode
        match std_listener.set_nonblocking(true) {
            Ok(s) => s,
            Err(_) => {
                return Err(ErrorType::SocketError(String::from(
                    "Problem when setting non blocking",
                )))
            }
        };

        // Convert the standard listener into a Tokio TcpListener
        return match TcpListener::from_std(std_listener) {
            Ok(l) => Ok(l),
            Err(_) => Err(ErrorType::SocketError(String::from(
                "Problem when converting tcp listener",
            ))),
        };
    }
}
