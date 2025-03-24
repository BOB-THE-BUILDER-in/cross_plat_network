use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, Mutex};
use tokio_tungstenite::{accept_async, tungstenite::protocol::Message};
use futures_util::{StreamExt, SinkExt};
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let addr = "127.0.0.1:8080";
    let listener = TcpListener::bind(addr).await.expect("Failed to bind");
    println!("WebSocket server running on ws://{}", addr);

    // Shared state: list of senders to broadcast messages
    let clients: Arc<Mutex<Vec<mpsc::UnboundedSender<Message>>>> = Arc::new(Mutex::new(Vec::new()));

    while let Ok((stream, _)) = listener.accept().await {
        let clients_clone = clients.clone();
        tokio::spawn(handle_connection(stream, clients_clone));
    }
}

async fn handle_connection(stream: TcpStream, clients: Arc<Mutex<Vec<mpsc::UnboundedSender<Message>>>>) {
    let addr = stream.peer_addr().expect("Connected streams should have a peer address");
    println!("New connection: {}", addr);

    let ws_stream = match accept_async(stream).await {
        Ok(ws) => ws,
        Err(e) => {
            eprintln!("Error during WebSocket handshake: {}", e);
            return;
        }
    };

    let (mut write, mut read) = ws_stream.split();
    let (tx, mut rx) = mpsc::unbounded_channel::<Message>();

    // Add this client's sender to the shared list
    {
        let mut clients_guard = clients.lock().await;
        clients_guard.push(tx);
    }

    // Task to forward messages from the broadcast channel to this client
    let write_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if let Err(e) = write.send(msg).await {
                eprintln!("Error sending to {}: {}", addr, e);
                break;
            }
        }
    });

    // Handle incoming messages and broadcast them
    while let Some(msg) = read.next().await {
        match msg {
            Ok(msg) if msg.is_text() || msg.is_binary() => {
                println!("Received from {}: {}", addr, msg);
                let clients_guard = clients.lock().await;
                for client_tx in clients_guard.iter() {
                    if let Err(e) = client_tx.send(msg.clone()) {
                        eprintln!("Failed to broadcast to a client: {}", e);
                    }
                }
            }
            Ok(msg) if msg.is_close() => {
                println!("Client disconnected: {}", addr);
                break;
            }
            Ok(_) => {} // Ignore other message types
            Err(e) => {
                eprintln!("Error receiving from {}: {}", addr, e);
                break;
            }
        }
    }

    // Remove this client from the list on disconnect
    {
        let mut clients_guard = clients.lock().await;
        clients_guard.retain(|client_tx| !client_tx.is_closed());
    }

    // Cancel the write task when the client disconnects
    write_task.abort();
}