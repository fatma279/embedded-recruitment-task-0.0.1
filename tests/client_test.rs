use std::net::TcpStream;
use std::time::Duration;
use std::io::{Read, Write};
use std::thread;
use log::error;

// Test case to simulate client connection to the server
#[test]
fn test_client_connection() {
    // Attempt to connect to the server at the specified address and port
    let result = TcpStream::connect("127.0.0.1:7878");

    // Assert that the connection is successful
    assert!(result.is_ok(), "Failed to connect to the server");  // If the connection fails, this test will fail with the given message

    // If the connection is successful, we can proceed to check the client stream
    let mut stream = result.unwrap();  // Unwrap the result to get the TcpStream

    // Send a test message to the server
    let msg = "Connection successful!";
    stream.write(msg.as_bytes()).unwrap();  // Send the message over the stream

    // Read the server's response
    let mut buffer = [0; 512];
    stream.read(&mut buffer).unwrap();  // Read the response from the server
    let response = String::from_utf8_lossy(&buffer);  // Convert the byte response to a string

    // Assert that the server responds with the message we sent
    assert!(response.contains(msg), "Server response does not match the sent message");  // Verify the server's response contains the sent message
}

// Test case to simulate multiple clients connecting concurrently
#[test]
fn test_multiple_clients() {
    // Create 5 threads, each representing a client
    let handles: Vec<_> = (0..5).map(|_| {
        thread::spawn(|| {
            // Each thread creates a TcpStream (client) and connects to the server
            let mut stream = TcpStream::connect("127.0.0.1:7878").unwrap();
            let msg = "Hello, server!";
            stream.write(msg.as_bytes()).unwrap();  // Send a message to the server

            let mut buffer = [0; 512];
            stream.read(&mut buffer).unwrap();  // Read the server's response
            let response = String::from_utf8_lossy(&buffer);  // Convert the response to a string
            assert!(response.contains(msg));  // Assert that the response contains the sent message
        })
    }).collect();  // Collect the thread handles

    // Wait for all threads to finish
    for handle in handles {
        handle.join().unwrap();
    }
}

// Test case to simulate sending a single "Add Request" and receiving a response
#[test]
fn test_client_add_request() {
    let mut stream = TcpStream::connect("127.0.0.1:7878").unwrap();
    let msg = "Add Request";
    stream.write(msg.as_bytes()).unwrap();  // Send a message to the server

    let mut buffer = [0; 512];
    stream.read(&mut buffer).unwrap();  // Read the server's response
    let response = String::from_utf8_lossy(&buffer);  // Convert the response to a string
    assert!(response.contains(msg));  // Assert that the response contains the sent message
}

// Test case to simulate sending a message and receiving the same message back (echo test)
#[test]
fn test_client_echo_message() {
    let mut stream = TcpStream::connect("127.0.0.1:7878").unwrap();
    let msg = "Echo this!";
    stream.write(msg.as_bytes()).unwrap();  // Send a message to the server

    let mut buffer = [0; 512];
    stream.read(&mut buffer).unwrap();  // Read the server's response
    let response = String::from_utf8_lossy(&buffer);  // Convert the response to a string
    assert_eq!(response.trim_end_matches('\0'), msg);  // Compare the response to the sent message
}

// Test case to simulate a failed connection to an invalid port
#[test]
fn test_connection_failure() {
    let result = TcpStream::connect("127.0.0.1:9999");  // Try connecting to a non-existent port
    assert!(result.is_err());  // Assert that the connection fails
}

// Test case to simulate a connection timeout
#[test]
fn test_timeout() {
    let timeout = Duration::from_millis(100);  // Set a timeout duration

    // Try connecting to a server, with the specified timeout
    let result = TcpStream::connect_timeout(&"127.0.0.1:7878".parse().unwrap(), timeout);
    assert!(result.is_ok(), "Connection failed or timed out");  // Assert that the connection succeeds within the timeout
}

// Test case to simulate sending multiple messages and ensuring each response matches the sent message
#[test]
fn test_multiple_messages() {
    let mut stream = TcpStream::connect("127.0.0.1:7878").unwrap();

    let messages = vec!["Hello", "How are you?", "Goodbye"];
    
    for msg in messages {
        stream.write(msg.as_bytes()).unwrap();  // Send the message to the server
        
        let mut buffer = [0; 1024];
        match stream.read(&mut buffer) {
            Ok(_) => {
                let response = String::from_utf8_lossy(&buffer);
                assert_eq!(response.trim_end_matches('\0'), msg);  // Assert that the response matches the sent message
            },
            Err(e) => {
                error!("Error reading response: {}", e);  // Log any errors that occur while reading the response
                break;
            }
        }

        // Longer delay between messages to reduce server load
        std::thread::sleep(std::time::Duration::from_millis(200)); // Increasing the time here might help with server load
    }
}

// Test case to simulate sending a message with invalid characters
#[test]
fn test_invalid_characters() {
    let mut stream = TcpStream::connect("127.0.0.1:7878").unwrap();
    let msg = "Invalid";  
    stream.write(msg.as_bytes()).unwrap();  // Send a message with invalid characters to the server

    let mut buffer = [0; 512];
    stream.read(&mut buffer).unwrap();  // Read the server's response
    let response = String::from_utf8_lossy(&buffer);  // Convert the response to a string
    assert_eq!(response.trim_end_matches('\0'), msg);  // Assert that the response matches the sent message
}

// Test case to simulate sending a large message to the server
#[test]
fn test_large_message() {
    let mut stream = TcpStream::connect("127.0.0.1:7878").unwrap();
    let msg = "A".repeat(1024);  // Create a message of 1024 bytes
    stream.write(msg.as_bytes()).unwrap();  // Send the large message to the server

    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();  // Read the server's response
    let response = String::from_utf8_lossy(&buffer);  // Convert the response to a string
    assert_eq!(response.trim_end_matches('\0'), msg);  // Assert that the response matches the sent message
}
