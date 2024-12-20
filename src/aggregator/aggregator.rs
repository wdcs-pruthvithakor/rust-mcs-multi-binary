use crate::utils;
use base64::{engine::general_purpose, Engine as _};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use futures::{SinkExt, StreamExt};
use serde_json::Value;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::sync::{Mutex, Notify};
use tokio_tungstenite::{accept_async, tungstenite::protocol::Message};

/// Aggregator process: Compute global average from signed client messages.
pub async fn aggregator_process(num_clients: usize, public_keys: Arc<Vec<VerifyingKey>>) {
    let averages = Arc::new(Mutex::new(Vec::new()));

    let is_ready = Arc::new(Mutex::new(false));
    let notify = Arc::new(Notify::new());

    let clients_verified: Arc<Mutex<usize>> = Arc::new(Mutex::new(0));
    let listener = TcpListener::bind("127.0.0.1:8080")
        .await
        .expect("Failed to bind to address");
    println!("Aggregator WebSocket server listening on ws://127.0.0.1:8080");
    while let Ok((stream, _)) = listener.accept().await {
        let public_keys = public_keys.clone();
        let average_list = averages.clone();
        let clients_verified_clone = clients_verified.clone();
        // let tx_clone = tx.clone();

        let is_ready_clone = is_ready.clone();
        let notify_clone = notify.clone();
        tokio::spawn(async move {
            let mut ws_stream: tokio_tungstenite::WebSocketStream<TcpStream> = accept_async(stream)
                .await
                .expect("Failed to accept WebSocket");
            println!("New client connected!");

            // Read messages from the WebSocket stream
            while let Some(msg) = ws_stream.next().await {
                match msg {
                    Ok(Message::Text(ref text)) if text == "receiver" => {
                        loop {
                            {
                                let ready = is_ready_clone.lock().await;
                                if *ready {
                                    break;
                                }
                            }
                            notify_clone.notified().await; // Wait to be notified
                        }
                        let mut avg_vec = average_list.lock().await;
                        let averages_copy = avg_vec.clone(); // Clone the data
                        avg_vec.clear();
                        let mut cv = clients_verified_clone.lock().await;
                        *cv = 0;
                        if let Some(global_avg) = utils::calculate_average(&averages_copy) {
                            println!("Aggregator: Global average BTC price: {:.4}", global_avg);
                            let response = format!("Global average BTC price: {:.4}", global_avg);
                            if let Err(e) = ws_stream.send(Message::Text(response)).await {
                                eprintln!("Failed to send message to client: {}", e);
                            }
                            // let _= stream.send(Message::Text(format!("Aggregator: Global average BTC price: {:.4}", global_avg)));
                            utils::save_global_data(&averages_copy, global_avg).unwrap_or_else(
                                |e| eprintln!("Aggregator: Failed to save global data: {e}"),
                            );
                        } else {
                            eprintln!("Aggregator: No valid averages received.");
                        }
                        let mut ready = is_ready_clone.lock().await;
                        *ready = false;
                    }
                    Ok(Message::Text(text)) => match process_message(&text, &public_keys).await {
                        Err(e) => eprintln!("Error processing message: {}", e),
                        Ok(avg) => {
                            let mut cv = clients_verified_clone.lock().await;
                            *cv += 1;
                            let mut avg_vec = average_list.lock().await;
                            avg_vec.push(avg);
                        }
                    },
                    Err(e) => {
                        eprintln!("WebSocket error: {}", e);
                    }
                    _ => {}
                }
            }
            let cv = clients_verified_clone.lock().await;
            if *cv >= num_clients {
                let mut ready = is_ready_clone.lock().await;
                *ready = true;
                notify_clone.notify_one();
            }
        });
    }
}

async fn process_message(
    msg: &str,
    public_keys: &[VerifyingKey],
) -> Result<f64, Box<dyn std::error::Error + Send>> {
    let data: Value = match serde_json::from_str(msg) {
        Ok(val) => val,
        Err(e) => return Err(Box::new(e) as Box<dyn std::error::Error + Send>),
    };

    let client_id = data["client_id"].as_u64().unwrap() as usize;
    let message = data["message"].as_str().unwrap();

    // Convert Vec<u8> to [u8; 64]
    let signature_vec = match general_purpose::STANDARD.decode(data["signature"].as_str().unwrap())
    {
        Ok(sig) => sig,
        Err(e) => return Err(Box::new(e) as Box<dyn std::error::Error + Send>),
    };

    let signature_array: &[u8; 64] = signature_vec
        .as_slice()
        .try_into()
        .expect("Invalid signature");

    let signature = Signature::from_bytes(signature_array);

    // Verify the signature using the client's public key
    if public_keys[client_id - 1]
        .verify(message.as_bytes(), &signature)
        .is_ok()
    {
        println!("Verified message from Client-{client_id}: {}", message);
    } else {
        eprintln!("Failed to verify signature for Client-{client_id}");
    }

    Ok(message.parse::<f64>().unwrap())
}
