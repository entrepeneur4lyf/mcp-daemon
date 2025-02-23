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

/// Stdio transport for server with json serialization
/// TODO: support for other binary serialzation formats
#[derive(Default, Clone)]
pub struct ServerStdioTransport;
#[async_trait]
impl Transport for ServerStdioTransport {
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

    async fn open(&self) -> Result<()> {
        Ok(())
    }

    async fn close(&self) -> Result<()> {
        Ok(())
    }
}

/// ClientStdioTransport launches a child process and communicates with it via stdio
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
