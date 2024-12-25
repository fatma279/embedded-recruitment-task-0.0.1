use std::io;
use std::sync::{Arc, Mutex};
use std::thread;
mod server;
use crate::server::Server;

fn main() -> io::Result<()> {
    // Using Arc and Mutex to share the object between threads
    let server = Arc::new(Mutex::new(Server::new("127.0.0.1:7878")?)); // Initialize server
    println!("Server running on 127.0.0.1:7878"); // Print server start message

    // Start the server in a separate thread
    let server_thread = {
        let server = Arc::clone(&server);
        thread::spawn(move || {
            let server = server.lock().unwrap(); // Lock the server to access it
            if let Err(e) = server.run() {
                eprintln!("Error running the server: {}", e);
            }
        })
    };

    // Simulate a duration to let the server handle requests
    thread::sleep(std::time::Duration::from_secs(60)); // Allow server to run for 60 seconds

    // Lock the server before calling stop to safely shut it down
    let server = server.lock().unwrap(); // Lock the server before calling stop
    server.stop(); // Call the stop method to shut down the server

    // Wait for the server thread to finish
    server_thread.join().unwrap();

    Ok(())
}
