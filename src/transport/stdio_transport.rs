//! Standard I/O (stdio) transport implementation for the Model Context Protocol
//!
//! This module provides transport implementations that use standard input and output
//! streams for communication between MCP clients and servers. It includes two main
//! transport types:
//!
//! - `ServerStdioTransport`: A simple transport that uses the process's stdin/stdout
//! - `ClientStdioTransport`: A transport that spawns a child process and communicates with it
//!
//! The stdio transport is particularly useful for:
//! - Command-line tools and applications
//! - Local process communication
//! - Testing and development
//! - Integration with shell scripts and other command-line utilities
//!
//! # Examples
//!
//! ## Server-side usage
//!
//! ```
//! use mcp_daemon::transport::{ServerStdioTransport, Transport};
//!
//! async fn example() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create a server transport
//!     let transport = ServerStdioTransport::default();
//!     
//!     // Open the transport
//!     transport.open().await?;
//!     
//!     // Receive a message
//!     if let Some(message) = transport.receive().await? {
//!         // Process the message
//!         println!("Received: {:?}", message);
//!         
//!         // Send a response
//!         transport.send(&message).await?;
//!     }
//!     
//!     // Close the transport
//!     transport.close().await?;
//!     
//!     Ok(())
//! }
//! ```
//!
//! ## Client-side usage
//!
//! ```
//! use mcp_daemon::transport::{ClientStdioTransport, Transport};
//!
//! async fn example() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create a client transport that spawns a server process
//!     let transport = ClientStdioTransport::new("mcp-server", &["--option", "value"])?;
//!     
//!     // Open the transport (spawns the process)
//!     transport.open().await?;
//!     
//!     // Send a message
//!     let message = serde_json::json!({
//!         "jsonrpc": "2.0",
//!         "method": "example",
//!         "params": {"hello": "world"},
//!         "id": 1
//!     });
//!     transport.send(&message).await?;
//!     
//!     // Receive a response
//!     if let Some(response) = transport.receive().await? {
//!         println!("Received: {:?}", response);
//!     }
//!     
//!     // Close the transport (terminates the process)
//!     transport.close().await?;
//!     
//!     Ok(())
//! }
//! ```

use super::{Message, Transport};
use super::Result;
use super::error::{TransportError, TransportErrorCode};
use async_trait::async_trait;
use std::io::{self, BufRead, Write};
use std::process::Stdio;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter};
use tokio::process::Child;
use tokio::sync::Mutex;
use tracing::debug;

/// Server-side stdio transport implementation
///
/// This transport uses the process's standard input and output streams for
/// communication. It's designed to be used in server applications that are
/// launched by a client and communicate through stdin/stdout.
///
/// The transport uses JSON serialization for messages, with each message
/// being a single line of JSON text followed by a newline character.
///
/// # Examples
///
/// ```
/// use mcp_daemon::transport::{ServerStdioTransport, Transport};
///
/// async fn run_server() -> Result<(), Box<dyn std::error::Error>> {
///     // Create a server transport
///     let transport = ServerStdioTransport::default();
///     
///     // Open the transport
///     transport.open().await?;
///     
///     // Main server loop
///     while let Some(message) = transport.receive().await? {
///         // Process the message
///         println!("Received: {:?}", message);
///         
///         // Send a response
///         transport.send(&message).await?;
///     }
///     
///     // Close the transport
///     transport.close().await?;
///     
///     Ok(())
/// }
/// ```
#[derive(Default, Clone)]
pub struct ServerStdioTransport;

#[async_trait]
impl Transport for ServerStdioTransport {
    /// Receives a message from the standard input
    ///
    /// This method reads a line from stdin, parses it as a JSON message,
    /// and returns the result. If the input stream is closed (EOF), it
    /// returns `Ok(None)`.
    ///
    /// # Returns
    ///
    /// * `Ok(Some(message))` - A message was successfully received
    /// * `Ok(None)` - The input stream was closed (EOF)
    /// * `Err(error)` - An error occurred while reading or parsing the message
    async fn receive(&self) -> Result<Option<Message>> {
        let stdin = io::stdin();
        let mut reader = stdin.lock();
        let mut line = String::new();
        reader.read_line(&mut line)?;
        if line.is_empty() {
            return Ok(None);
        }

        debug!("Received: {line}");
        let message: Message = serde_json::from_str(&line)?;
        Ok(Some(message))
    }

    /// Sends a message to the standard output
    ///
    /// This method serializes the message to JSON, writes it to stdout
    /// followed by a newline character, and flushes the output.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to send
    ///
    /// # Returns
    ///
    /// * `Ok(())` - The message was successfully sent
    /// * `Err(error)` - An error occurred while serializing or writing the message
    async fn send(&self, message: &Message) -> Result<()> {
        let stdout = io::stdout();
        let mut writer = stdout.lock();
        let serialized = serde_json::to_string(message)?;
        debug!("Sending: {serialized}");
        writer.write_all(serialized.as_bytes())?;
        writer.write_all(b"\n")?;
        writer.flush()?;
        Ok(())
    }

    /// Opens the transport
    ///
    /// For the server transport, this is a no-op since the stdin/stdout
    /// streams are already available.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - The transport was successfully opened
    async fn open(&self) -> Result<()> {
        Ok(())
    }

    /// Closes the transport
    ///
    /// For the server transport, this is a no-op since the stdin/stdout
    /// streams are managed by the process.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - The transport was successfully closed
    async fn close(&self) -> Result<()> {
        Ok(())
    }
}

/// Client-side stdio transport implementation
///
/// This transport spawns a child process and communicates with it through
/// its standard input and output streams. It's designed to be used in client
/// applications that need to launch and communicate with a server process.
///
/// The transport uses JSON serialization for messages, with each message
/// being a single line of JSON text followed by a newline character.
///
/// # Examples
///
/// ```
/// use mcp_daemon::transport::{ClientStdioTransport, Transport};
///
/// async fn run_client() -> Result<(), Box<dyn std::error::Error>> {
///     // Create a client transport that spawns a server process
///     let transport = ClientStdioTransport::new("mcp-server", &["--option", "value"])?;
///     
///     // Open the transport (spawns the process)
///     transport.open().await?;
///     
///     // Send a message
///     let message = serde_json::json!({
///         "jsonrpc": "2.0",
///         "method": "example",
///         "params": {"hello": "world"},
///         "id": 1
///     });
///     transport.send(&message).await?;
///     
///     // Receive a response
///     if let Some(response) = transport.receive().await? {
///         println!("Received: {:?}", response);
///     }
///     
///     // Close the transport (terminates the process)
///     transport.close().await?;
///     
///     Ok(())
/// }
/// ```
#[derive(Clone)]
pub struct ClientStdioTransport {
    stdin: Arc<Mutex<Option<BufWriter<tokio::process::ChildStdin>>>>,
    stdout: Arc<Mutex<Option<BufReader<tokio::process::ChildStdout>>>>,
    child: Arc<Mutex<Option<Child>>>,
    program: String,
    args: Vec<String>,
}

impl ClientStdioTransport {
    /// Creates a new stdio transport by spawning a program with the given arguments
    ///
    /// This constructor creates a new transport that will spawn the specified
    /// program with the given arguments when `open()` is called. The transport
    /// will communicate with the program through its stdin/stdout streams.
    ///
    /// # Arguments
    ///
    /// * `program` - The program to spawn
    /// * `args` - The arguments to pass to the program
    ///
    /// # Returns
    ///
    /// * `Ok(transport)` - The transport was successfully created
    /// * `Err(error)` - An error occurred while creating the transport
    ///
    /// # Examples
    ///
    /// ```
    /// use mcp_daemon::transport::ClientStdioTransport;
    ///
    /// // Create a transport that will spawn the "mcp-server" program
    /// let transport = ClientStdioTransport::new("mcp-server", &["--option", "value"]).unwrap();
    /// ```
    pub fn new(program: &str, args: &[&str]) -> Result<Self> {
        Ok(ClientStdioTransport {
            stdin: Arc::new(Mutex::new(None)),
            stdout: Arc::new(Mutex::new(None)),
            child: Arc::new(Mutex::new(None)),
            program: program.to_string(),
            args: args.iter().map(|&s| s.to_string()).collect(),
        })
    }
}

#[async_trait]
impl Transport for ClientStdioTransport {
    /// Receives a message from the child process
    ///
    /// This method reads a line from the child process's stdout, parses it
    /// as a JSON message, and returns the result. If the output stream is
    /// closed (EOF), it returns `Ok(None)`.
    ///
    /// # Returns
    ///
    /// * `Ok(Some(message))` - A message was successfully received
    /// * `Ok(None)` - The output stream was closed (EOF)
    /// * `Err(error)` - An error occurred while reading or parsing the message
    async fn receive(&self) -> Result<Option<Message>> {
        debug!("ClientStdioTransport: Starting to receive message");
        let mut stdout = self.stdout.lock().await;
        let stdout = stdout
            .as_mut()
            .ok_or_else(|| TransportError::new(TransportErrorCode::InvalidState, "Transport not opened"))?;

        let mut line = String::new();
        debug!("ClientStdioTransport: Reading line from process");
        let bytes_read = stdout.read_line(&mut line).await?;
        debug!("ClientStdioTransport: Read {} bytes", bytes_read);

        if bytes_read == 0 {
            debug!("ClientStdioTransport: Received EOF from process");
            return Ok(None);
        }
        debug!("ClientStdioTransport: Received from process: {line}");
        let message: Message = serde_json::from_str(&line)?;
        debug!("ClientStdioTransport: Successfully parsed message");
        Ok(Some(message))
    }

    /// Sends a message to the child process
    ///
    /// This method serializes the message to JSON, writes it to the child
    /// process's stdin followed by a newline character, and flushes the output.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to send
    ///
    /// # Returns
    ///
    /// * `Ok(())` - The message was successfully sent
    /// * `Err(error)` - An error occurred while serializing or writing the message
    async fn send(&self, message: &Message) -> Result<()> {
        debug!("ClientStdioTransport: Starting to send message");
        let mut stdin = self.stdin.lock().await;
        let stdin = stdin
            .as_mut()
            .ok_or_else(|| TransportError::new(TransportErrorCode::InvalidState, "Transport not opened"))?;

        let serialized = serde_json::to_string(message)?;
        debug!("ClientStdioTransport: Sending to process: {serialized}");
        stdin.write_all(serialized.as_bytes()).await?;
        stdin.write_all(b"\n").await?;
        stdin.flush().await?;
        debug!("ClientStdioTransport: Successfully sent and flushed message");
        Ok(())
    }

    /// Opens the transport
    ///
    /// This method spawns the child process and sets up the stdin/stdout
    /// streams for communication.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - The transport was successfully opened
    /// * `Err(error)` - An error occurred while spawning the process or setting up the streams
    async fn open(&self) -> Result<()> {
        debug!("ClientStdioTransport: Opening transport");
        let mut child = tokio::process::Command::new(&self.program)
            .args(&self.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;

        debug!("ClientStdioTransport: Child process spawned");
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| TransportError::new(TransportErrorCode::ConnectionFailed, "Child process stdin not available"))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| TransportError::new(TransportErrorCode::ConnectionFailed, "Child process stdout not available"))?;

        *self.stdin.lock().await = Some(BufWriter::new(stdin));
        *self.stdout.lock().await = Some(BufReader::new(stdout));
        *self.child.lock().await = Some(child);

        Ok(())
    }

    /// Closes the transport
    ///
    /// This method performs a graceful shutdown of the child process:
    /// 1. Flushes and closes the stdin stream
    /// 2. Waits for the process to exit gracefully (with a timeout)
    /// 3. If the process doesn't exit, sends SIGTERM
    /// 4. If the process still doesn't exit, forces a kill
    ///
    /// # Returns
    ///
    /// * `Ok(())` - The transport was successfully closed
    /// * `Err(error)` - An error occurred while closing the transport
    async fn close(&self) -> Result<()> {
        const GRACEFUL_TIMEOUT_MS: u64 = 1000;
        const SIGTERM_TIMEOUT_MS: u64 = 500;
        debug!("Starting graceful shutdown");
        {
            let mut stdin_guard = self.stdin.lock().await;
            if let Some(stdin) = stdin_guard.as_mut() {
                debug!("Flushing stdin");
                stdin.flush().await?;
            }
            *stdin_guard = None;
        }

        let mut child_guard = self.child.lock().await;
        let Some(child) = child_guard.as_mut() else {
            debug!("No child process to close");
            return Ok(());
        };

        debug!("Attempting graceful shutdown");
        match child.try_wait()? {
            Some(status) => {
                debug!("Process already exited with status: {}", status);
                *child_guard = None;
                return Ok(());
            }
            None => {
                debug!("Waiting for process to exit gracefully");
                tokio::time::sleep(tokio::time::Duration::from_millis(GRACEFUL_TIMEOUT_MS)).await;
            }
        }

        if child.try_wait()?.is_none() {
            debug!("Process still running, sending SIGTERM");
            child.kill().await?;
            tokio::time::sleep(tokio::time::Duration::from_millis(SIGTERM_TIMEOUT_MS)).await;
        }

        if child.try_wait()?.is_none() {
            debug!("Process not responding to SIGTERM, forcing kill");
            child.kill().await?;
        }

        match child.wait().await {
            Ok(status) => debug!("Process exited with status: {}", status),
            Err(e) => debug!("Error waiting for process exit: {}", e),
        }

        *child_guard = None;
        debug!("Shutdown complete");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::transport::{JsonRpcMessage, JsonRpcRequest, JsonRpcVersion};

    use super::*;
    use std::time::Duration;
    #[tokio::test]
    #[cfg(unix)]
    async fn test_stdio_transport() -> Result<()> {
        // Create transport connected to cat command which will stay alive
        let transport = ClientStdioTransport::new("cat", &[])?;

        // Create a test message
        let test_message = JsonRpcMessage::Request(JsonRpcRequest {
            id: 1,
            method: "test".to_string(),
            params: Some(serde_json::json!({"hello": "world"})),
            jsonrpc: JsonRpcVersion::default(),
        });

        // Open transport
        transport.open().await?;

        // Send message
        transport.send(&test_message).await?;

        // Receive echoed message
        let response = transport.receive().await?;

        // Verify the response matches
        assert_eq!(Some(test_message), response);

        // Clean up
        transport.close().await?;

        Ok(())
    }

    #[tokio::test]
    #[cfg(unix)]
    async fn test_graceful_shutdown() -> Result<()> {
        // Create transport with a sleep command that runs for 5 seconds

        let transport = ClientStdioTransport::new("sleep", &["5"])?;
        transport.open().await?;

        // Spawn a task that will read from the transport
        let transport_clone = transport.clone();
        let read_handle = tokio::spawn(async move {
            let result = transport_clone.receive().await;
            debug!("Receive returned: {:?}", result);
            result
        });

        // Wait a bit to ensure the process is running
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Initiate graceful shutdown
        let start = std::time::Instant::now();
        transport.close().await?;
        let shutdown_duration = start.elapsed();

        // Verify that:
        // 1. The read operation was cancelled (returned None)
        // 2. The shutdown completed in less than 5 seconds (didn't wait for sleep)
        // 3. The process was properly terminated
        let read_result = read_handle.await?;
        assert!(read_result.is_ok());
        assert_eq!(read_result.unwrap(), None);
        assert!(shutdown_duration < Duration::from_secs(5));

        // Verify process is no longer running
        let child_guard = transport.child.lock().await;
        assert!(child_guard.is_none());

        Ok(())
    }

    #[tokio::test]
    #[cfg(unix)]
    async fn test_shutdown_with_pending_io() -> Result<()> {
        // Use 'cat' command which will echo input
        let transport = ClientStdioTransport::new("cat", &[])?;
        transport.open().await?;

        // Start a receive operation that will be pending
        let transport_clone = transport.clone();
        let read_handle = tokio::spawn(async move { transport_clone.receive().await });

        // Give some time for read operation to start
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Send a message
        let test_message = JsonRpcMessage::Request(JsonRpcRequest {
            id: 1,
            method: "test".to_string(),
            params: Some(serde_json::json!({"hello": "world"})),
            jsonrpc: JsonRpcVersion::default(),
        });
        transport.send(&test_message).await?;

        // Wait a bit to ensure the message is processed
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Initiate shutdown
        transport.close().await?;

        // Verify the read operation completed successfully
        let read_result = read_handle.await?;
        assert!(read_result.is_ok());
        assert_eq!(read_result.unwrap(), Some(test_message));

        Ok(())
    }
}
