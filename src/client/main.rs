use mcs_binary::utils;
use tokio::task;
mod client;

#[tokio::main]
async fn main() {
    // let num_clients: usize = 5;
    let matches = utils::parse_arguments();

    // Extract the mode and times arguments
    // let mode = if let Some(mode) = matches.get_one::<String>("mode") {
    //     mode.as_str()
    // } else {
    //     ""
    // };
    // let keypairs = utils::generate_keypairs(num_clients);
    let keypairs = utils::load_private_keys("client_keys.json");
    // Save keys locally (optional)
    // utils::save_keys(keypairs.clone(), "client_keys.json").expect("Failed to save keys");

    // let keypair = keypairs[0].clone();
    // let ws_url = "ws://127.0.0.1:8080";

    // // Connect to the WebSocket server
    // let (ws_stream, _) = connect_async(ws_url)
    //     .await
    //     .expect("Failed to connect WebSocket");
    // println!("Connected to WebSocket server at {}", ws_url);
    // let (mut write, _) = ws_stream.split();

    // let id = 1; // Assign a unique client ID
    // let message = format!("Client-{id}");
    // let signed_message = keypair.sign(message.as_bytes());

    // let serialized_data = json!({
    //     "client_id": id,
    //     "message": message,
    //     "signature": general_purpose::STANDARD.encode(signed_message.to_bytes())
    // });

    // // Periodically send signed data to the server
    // for _ in 0..10 {
    //     write
    //         .send(Message::Text(serialized_data.to_string()))
    //         .await
    //         .expect("Failed to send message");
    //     tokio::time::sleep(Duration::from_secs(1)).await;
    // }

    let default_mode = String::default();
    let times: u64 = matches
            .get_one::<String>("times")
            .unwrap_or(&default_mode)
            .parse()
            .unwrap_or_else(|_|{ eprintln!("Failed to parse input time value, please enter valid seconds, taking default 1 to calculate.."); 1});

    let mut clients = Vec::new();
    for (id, keypair) in keypairs.into_iter().enumerate() {
        clients.push(task::spawn(client::client_process(id + 1, keypair, times)));
    }
    println!("Will listen for {} seconds.", times);
    clients.push(task::spawn(client::get_results()));
    for client in clients {
        let _ = client.await;
    }
}
