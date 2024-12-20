use crate::utils;
use base64::{
    alphabet,
    engine::{self, general_purpose},
    Engine as _,
};
use ed25519_dalek::{Signature, Signer, SigningKey};
use futures::SinkExt;
use futures::StreamExt;
use serde_json::json;
use serde_json::Value;
use tokio::time::{timeout, Duration, Instant};
use tokio::{net::TcpStream, sync::mpsc};
use tokio_tungstenite::{
    connect_async, tungstenite::protocol::frame::coding::CloseCode,
    tungstenite::protocol::CloseFrame, tungstenite::protocol::Message, MaybeTlsStream,
};

/// Connect to WebSocket server.
async fn connect_to_websocket(
) -> Result<tokio_tungstenite::WebSocketStream<MaybeTlsStream<TcpStream>>, Box<dyn std::error::Error>>
{
    let url = "wss://stream.binance.com:9443/ws/btcusdt@trade";
    let (ws_stream, _) = connect_async(url).await?;
    Ok(ws_stream)
}

/// Client process: Fetch prices, calculate average, sign, and send to aggregator.
pub async fn client_process(id: usize, keypair: SigningKey, duration: u64) {
    let mut ws_stream = match connect_to_websocket().await {
        Ok(ws) => ws,
        Err(e) => {
            eprintln!("Client {id}: Failed to connect to WebSocket: {e}");
            return;
        }
    };

    println!("Client {id}: Connected to WebSocket.");
    let mut prices: Vec<f64> = Vec::new();
    let start_time = Instant::now();

    while start_time.elapsed().as_secs() < duration {
        let remaining_time = duration.saturating_sub(start_time.elapsed().as_secs());

        // Set a timeout for receiving a message
        let result = timeout(Duration::from_secs(remaining_time), ws_stream.next()).await;

        match result {
            Ok(Some(Ok(Message::Text(text)))) => {
                if let Ok(price) = utils::process_message(&text) {
                    prices.push(price);
                }
            }
            Ok(Some(Err(e))) => {
                eprintln!("Client {id}: WebSocket error: {e}");
                break;
            }
            Ok(None) => {
                eprintln!("Client {id}: WebSocket stream closed.");
                break;
            }
            Err(_) => {
                eprintln!("Client {id}: Timeout reached while waiting for WebSocket message.");
                break;
            }
            _ => {
                break;
            }
        }
    }
    let ws_url = "ws://127.0.0.1:8080";

    // Connect to the WebSocket server
    let (ws_stream, _) = connect_async(ws_url)
        .await
        .expect("Failed to connect WebSocket");
    println!("Connected to WebSocket server at {}", ws_url);
    let (mut write, _) = ws_stream.split();

    // let message = format!("Client-{id}");
    // let signed_message = keypair.sign(message.as_bytes());

    if let Some(avg) = utils::calculate_average(&prices) {
        println!("Client {id}: Average BTC price: {:.4}", avg);

        let message = format!("{}", avg);
        let signature = keypair.sign(message.as_bytes());
        let serialized_data = json!({
            "client_id": id,
            "message": message,
            "signature": general_purpose::STANDARD.encode(signature.to_bytes())
        });
        write
            .send(Message::Text(serialized_data.to_string()))
            .await
            .expect("Failed to send message");
        utils::save_client_data(id, &prices, avg)
            .unwrap_or_else(|e| eprintln!("Client {id}: Failed to save data: {e}"));
        let close_frame = CloseFrame {
            code: CloseCode::Normal, // Normal closure
            reason: std::borrow::Cow::Borrowed("Closing the connection gracefully"),
        };

        // Send the close frame to the server
        if let Err(e) = write.send(Message::Close(Some(close_frame))).await {
            eprintln!("Failed to send close frame: {e}");
        } else {
            println!("Server sent close frame.");
        }

        // // Make sure the WebSocket is fully closed after sending the close frame
        // ws_stream.await.unwrap_or_else(|e| eprintln!("Client {id}: Error while closing WebSocket: {e}"));
    } else {
        eprintln!("Client {id}: No data points collected.");
    }
}

pub async fn get_results() {
    let ws_url = "ws://127.0.0.1:8080";

    // Connect to the WebSocket server
    let (ws_stream, _) = connect_async(ws_url)
        .await
        .expect("Failed to connect WebSocket");
    println!(
        "Connected to WebSocket server at {} for final client",
        ws_url
    );
    let (mut write, mut read) = ws_stream.split();
    write
        .send(Message::Text(format!("receiver")))
        .await
        .expect("Failed to send message");
    if let Some(message) = read.next().await {
        match message {
            Ok(Message::Text(text)) => {
                // When a text message is received, print the content
                // You could parse it if needed, e.g., to extract the global average
                println!("Received from server: {}", text);

                // // Optionally, parse the message if it's JSON (e.g., for further processing)
                // match serde_json::from_str::<Value>(&text) {
                //     Ok(data) => {
                //         if let Some(global_avg) = data["global_avg"].as_f64() {
                //             println!("Global average BTC price: {:.4}", global_avg);
                //         }
                //     }
                //     Err(_) => eprintln!("Failed to parse server response as JSON."),
                // }
            }
            Ok(Message::Close(_)) => {
                println!("Server closed the connection.");
            }
            Err(e) => {
                eprintln!("WebSocket error: {}", e);
            }
            _ => {} // Ignore other types of messages
        }
    }
    let close_frame = CloseFrame {
        code: CloseCode::Normal, // Normal closure
        reason: std::borrow::Cow::Borrowed("Closing the connection gracefully"),
    };

    // Send the close frame to the server
    if let Err(e) = write.send(Message::Close(Some(close_frame))).await {
        eprintln!("Failed to send close frame: {e}");
    } else {
        println!("Server sent close frame.");
    }
}
