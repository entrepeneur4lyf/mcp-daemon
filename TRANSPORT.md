```mermaid
flowchart TD
    Transport[Transport Trait] --> ServerTransport[Server-side Transports]
    Transport --> ClientTransport[Client-side Transports]
    
    ServerTransport --> ServerStdio[Server Stdio Transport]
    ServerTransport --> ServerInMemory[Server InMemory Transport]
    ServerTransport --> ServerHttp[Server HTTP Transports]
    
    ClientTransport --> ClientStdio[Client Stdio Transport]
    ClientTransport --> ClientInMemory[Client InMemory Transport]
    ClientTransport --> ClientHttp[Client HTTP Transports]
    
    ServerHttp --> ServerSse[Server SSE Transport]
    ServerHttp --> ServerWs[Server WebSocket Transport]
    
    ClientHttp --> ClientWs[Client WebSocket Transport]
    
    subgraph Transport Interface
        direction TB
        SendMethod["async fn send(&self, message: &Message) -> Result<()>"]
        ReceiveMethod["async fn receive(&self) -> Result<Option<Message>>"]
        OpenMethod["async fn open(&self) -> Result<()>"]
        CloseMethod["async fn close(&self) -> Result<()>"]
    end
    
    Transport -.-> Transport Interface
    
    subgraph Error Handling
        direction TB
        ConnectionErrors[Connection Errors]
        MessageErrors[Message Errors]
        ProtocolErrors[Protocol Errors]
        TransportErrors[Transport Errors]
        SessionErrors[Session Errors]
        WebSocketErrors[WebSocket Errors]
        SseErrors[SSE Errors]
    end
    
    subgraph Message Format
        direction TB
        JsonRpcRequest[JSON-RPC Request]
        JsonRpcResponse[JSON-RPC Response]
        JsonRpcNotification[JSON-RPC Notification]
    end
    
    Transport -.-> Error Handling
    Transport -.-> Message Format
```
