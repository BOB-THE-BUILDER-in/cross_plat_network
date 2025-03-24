use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use futures_util::{StreamExt, SinkExt};

#[tokio::main]
async fn main() {
    let url = "ws://127.0.0.1:8080";
    // Use &str directly for connect_async, as it implements IntoClientRequest
    let (ws_stream, _) = connect_async(url)
        .await
        .expect("Failed to connect");
    println!("Connected to {}", url);

    let (mut write, mut read) = ws_stream.split();

    // Spawn a task to send messages
    tokio::spawn(async move {
        let messages = vec!["Hello", "from", "Rust", "client!"];
        for msg in messages {
            // Convert String to Utf8Bytes for Message::Text
            write
                .send(Message::Text(msg.to_string().into()))
                .await
                .expect("Failed to send message");
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    });

    // Handle incoming messages
    while let Some(msg) = read.next().await {
        match msg {
            Ok(msg) => println!("Received: {}", msg),
            Err(e) => {
                eprintln!("Error: {}", e);
                break;
            }
        }
    }
}