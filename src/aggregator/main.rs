use mcs_binary::utils;
use std::sync::Arc;
mod aggregator;

#[tokio::main]
async fn main() {
    let num_clients = 5; // Adjust for multiple client instances
    let keypairs = utils::generate_keypairs(num_clients);
    utils::save_keys(keypairs.clone(), "client_keys.json").expect("Failed to save keys");
    let public_keys = utils::load_public_keys("client_keys.json");
    let public_keys = Arc::new(public_keys);

    aggregator::aggregator_process(num_clients, public_keys).await;
}
