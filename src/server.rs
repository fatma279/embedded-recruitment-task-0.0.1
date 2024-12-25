use log::{error, info};
use std::{
    io::{self, ErrorKind, Read, Write},
    net::{TcpListener, TcpStream},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};
use threadpool::ThreadPool;

pub struct Client {
    stream: TcpStream,
}

impl Client {
    pub fn new(stream: TcpStream) -> Self {
        Client { stream }
    }

    pub fn handle(&mut self) -> io::Result<()> {
        let mut buffer = [0; 4096];
        // Read data from the client
        let bytes_read = self.stream.read(&mut buffer)?;

        if bytes_read == 0 {
            info!("Client disconnected."); // Log when the client disconnects
            self.stream.shutdown(std::net::Shutdown::Both)?; // Properly close the connection
            return Ok(());
        }

        let received_data = String::from_utf8_lossy(&buffer[..bytes_read]);
        info!("Received: {}", received_data);

        // If the data cannot be converted to valid UTF-8 text
        if received_data.is_empty() {
            error!("Received invalid UTF-8 data");
            return Ok(()); // Connection can be closed if data is invalid
        }

        self.stream.write_all(received_data.as_bytes())?;
        self.stream.flush()?;

        Ok(())
    }
}

pub struct Server {
    listener: TcpListener,
    is_running: Arc<AtomicBool>,
    thread_pool: ThreadPool,
}

impl Server {
    /// Creates a new server instance
    pub fn new(addr: &str) -> io::Result<Self> {
        let pool_size = 4; // Set the thread pool size
        let listener = TcpListener::bind(addr)?;
        let is_running = Arc::new(AtomicBool::new(false));
        let thread_pool = ThreadPool::new(pool_size);
        Ok(Server {
            listener,
            is_running,
            thread_pool,
        })
    }

    /// Runs the server, listening for incoming connections and handling them
    pub fn run(&self) -> io::Result<()> {
        self.is_running.store(true, Ordering::SeqCst); // Set the server as running
        info!("Server is running on {}", self.listener.local_addr()?);
        self.listener.set_nonblocking(true)?; // Set listener to non-blocking mode

        while self.is_running.load(Ordering::SeqCst) {
            match self.listener.accept() {
                Ok((stream, addr)) => {
                    info!("New client connected: {}", addr); // Log new client connection
                    let is_running = Arc::clone(&self.is_running);
                    self.thread_pool.execute(move || {
                        // Handle the client request
                        let mut client = Client::new(stream);
                        while is_running.load(Ordering::SeqCst) {
                            if let Err(e) = client.handle() {
                                error!("Error handling client: {}", e); // Log error if client handling fails
                                break;
                            }
                        }
                    });
                }
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                    // No incoming connections, sleep briefly to reduce CPU usage
                    thread::sleep(Duration::from_millis(50)); 
                }
                Err(e) => {
                    error!("Error accepting connection: {}", e); // Log other errors that occur while accepting connections
                }
            }
        }

        info!("Server stopped."); // Log when the server stops
        Ok(())
    }
    
     /// Stops the server by setting the `is_running` flag to `false`
    pub fn stop(&self) {
        if self.is_running.load(Ordering::SeqCst) {
            self.is_running.store(false, Ordering::SeqCst);
            info!("Shutdown signal sent."); // Log when shutdown is initiated
        } else {
            info!("Server was already stopped or not running."); // Log if the server was already stopped
        }
    }
}
