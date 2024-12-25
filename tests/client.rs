use embedded_recruitment_task::message::{client_message, ServerMessage};
use log::error;
use log::info;
use prost::Message;
use std::io::Read;
use std::io::Write;
use std::{
    io,
    net::{SocketAddr, TcpStream, ToSocketAddrs},
    time::Duration,
};

// TCP/IP Client struct
pub struct Client {
    ip: String,         // IP address of the server
    port: u32,          // Port of the server
    timeout: Duration,  // Connection timeout duration
    stream: Option<TcpStream>, // Optional TcpStream, holds the active connection
}

impl Client {
    // Constructor for creating a new Client instance
    pub fn new(ip: &str, port: u32, timeout_ms: u64) -> Self {
        Client {
            ip: ip.to_string(),
            port,
            timeout: Duration::from_millis(timeout_ms),
            stream: None, // Initially no connection
        }
    }

    // Connect the client to the server
    pub fn connect(&mut self) -> io::Result<()> {
        println!("Connecting to {}:{}", self.ip, self.port);

        // Resolve the address
        let address = format!("{}:{}", self.ip, self.port);
        let socket_addrs: Vec<SocketAddr> = address.to_socket_addrs()?.collect();

        // If no valid socket address is found, return an error
        if socket_addrs.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Invalid IP or port",
            ));
        }

        // Connect to the server with a timeout
        let stream = TcpStream::connect_timeout(&socket_addrs[0], self.timeout)?;
        self.stream = Some(stream);  // Save the active connection

        println!("Connected to the server!");
        Ok(())
    }

    // Disconnect the client
    pub fn disconnect(&mut self) -> io::Result<()> {
        // If there's an active connection, shut it down
        if let Some(stream) = self.stream.take() {
            stream.shutdown(std::net::Shutdown::Both)?;  // Close both read and write channels
        }

        println!("Disconnected from the server!");
        Ok(())
    }

    // generic message to send message to the server
    pub fn send(&mut self, message: client_message::Message) -> io::Result<()> {
        // Check if there is an active connection
        if let Some(ref mut stream) = self.stream {
            // Encode the message into a buffer using Protocol Buffers
            let mut buffer = Vec::new();
            message.encode(&mut buffer);

            // Send the buffer to the server
            stream.write_all(&buffer)?;  // Send the encoded message
            stream.flush()?;  // Ensure all data is sent

            println!("Sent message: {:?}", message);
            Ok(())
        } else {
            // If no active connection, return an error
            Err(io::Error::new(
                io::ErrorKind::NotConnected,
                "No active connection",
            ))
        }
    }

    // Receive a message from the server
    pub fn receive(&mut self) -> io::Result<ServerMessage> {
        // Check if there is an active connection
        if let Some(ref mut stream) = self.stream {
            info!("Receiving message from the server");

            // Read the incoming data into a buffer
            let mut buffer = vec![0u8; 1024];
            let bytes_read = stream.read(&mut buffer)?;

            // If no bytes are read, the server disconnected
            if bytes_read == 0 {
                info!("Server disconnected.");
                return Err(io::Error::new(
                    io::ErrorKind::ConnectionAborted,
                    "Server disconnected",
                ));
            }

            info!("Received {} bytes from the server", bytes_read);

            // Decode the received message from the buffer
            ServerMessage::decode(&buffer[..bytes_read]).map_err(|e| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Failed to decode ServerMessage: {}", e),
                )
            })
        } else {
            // If no active connection, return an error
            error!("No active connection");
            Err(io::Error::new(
                io::ErrorKind::NotConnected,
                "No active connection",
            ))
        }
    }
}
