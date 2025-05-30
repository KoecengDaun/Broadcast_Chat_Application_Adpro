use futures_util::sink::SinkExt;
use futures_util::stream::StreamExt;
use std::error::Error;
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast::{channel, Sender};
use tokio_websockets::{Message, ServerBuilder, WebSocketStream};

async fn handle_connection(
    addr: SocketAddr,
    mut ws_stream: WebSocketStream<TcpStream>,
    bcast_tx: Sender<String>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut bcast_rx = bcast_tx.subscribe();
    
    loop {
        tokio::select! {
            result = ws_stream.next() => {
                match result {
                    Some(Ok(msg)) => {
                        if let Some(text) = msg.as_text() {
                            println!("Received from {}: {}", addr, text);
                            let _ = bcast_tx.send(format!("{}: {}", addr, text));
                        }
                    },
                    Some(Err(e)) => {
                        println!("Error receiving message from {}: {}", addr, e);
                        break;
                    },
                    None => {
                        println!("Connection closed by {}", addr);
                        break;
                    }
                }
            },
            
            result = bcast_rx.recv() => {
                match result {
                    Ok(msg) => {
                        if !msg.starts_with(&format!("{}: ", addr)) {
                            if let Err(e) = ws_stream.send(Message::text(msg)).await {
                                println!("Error sending message to {}: {}", addr, e);
                                break;
                            }
                        }
                    },
                    Err(e) => {
                        println!("Broadcast channel error: {}", e);
                        break;
                    }
                }
            }
        }
    }
    
    println!("Client {} disconnected", addr);
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let (bcast_tx, _) = channel(16);

    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("listening on port 8080");

    loop {
        let (socket, addr) = listener.accept().await?;
        println!("New connection from {addr:?}");
        let bcast_tx = bcast_tx.clone();
        tokio::spawn(async move {
            let (_req, ws_stream) = ServerBuilder::new().accept(socket).await?;

            handle_connection(addr, ws_stream, bcast_tx).await
        });
    }
}