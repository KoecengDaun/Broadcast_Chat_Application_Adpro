use futures_util::stream::StreamExt;
use futures_util::SinkExt;
use http::Uri;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio_websockets::{ClientBuilder, Message};

#[tokio::main]
async fn main() -> Result<(), tokio_websockets::Error> {
    let (mut ws_stream, _) =
        ClientBuilder::from_uri(Uri::from_static("ws://127.0.0.1:8080"))
            .connect()
            .await?;

    let stdin = tokio::io::stdin();
    let mut stdin = BufReader::new(stdin).lines();

    println!("Connected to chat server. Type a message and press Enter to send.");
    
    loop {
        tokio::select! {
            line = stdin.next_line() => {
                match line {
                    Ok(Some(text)) => {
                        if text.trim().is_empty() {
                            continue;
                        }
                        
                        if let Err(e) = ws_stream.send(Message::text(text)).await {
                            eprintln!("Error sending message: {}", e);
                            break;
                        }
                    },
                    Ok(None) => {
                        println!("Stdin closed, exiting...");
                        break;
                    },
                    Err(e) => {
                        eprintln!("Error reading from stdin: {}", e);
                        break;
                    }
                }
            },

            result = ws_stream.next() => {
                match result {
                    Some(Ok(message)) => {
                        if let Some(text) = message.as_text() {
                            println!("{}", text);
                        }
                    },
                    Some(Err(e)) => {
                        eprintln!("Error receiving message: {}", e);
                        break;
                    },
                    None => {
                        println!("Server closed the connection");
                        break;
                    }
                }
            }
        }
    }
    
    let _ = ws_stream.close().await;
    
    Ok(())
}