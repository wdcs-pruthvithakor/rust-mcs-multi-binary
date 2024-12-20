use mcs_binary::utils;
use tokio::task;
mod client;

#[tokio::main]
async fn main() {
    let matches = utils::parse_arguments();

    let keypairs = utils::load_private_keys("client_keys.json");

    let default_mode = String::default();
    let mode = matches
        .get_one::<String>("mode")
        .unwrap_or(&default_mode)
        .as_str();
    let times: u64 = matches
            .get_one::<String>("times")
            .unwrap_or(&default_mode)
            .parse()
            .unwrap_or_else(|_|{ eprintln!("Failed to parse input time value, please enter valid seconds, taking default 1 to calculate.."); 1});
    match mode {
        "cache" => {
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
        "read" => utils::read_mode(keypairs.len()).expect("Failed to read price data"),
        _ => eprintln!("Invalid mode: {mode}. Use --mode=cache or --mode=read."),
    };
}
