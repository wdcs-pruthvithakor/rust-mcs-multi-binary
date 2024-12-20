use base64::{engine::general_purpose, Engine as _};
use clap::{Arg, Command};
use ed25519_dalek::{SigningKey, VerifyingKey};
use rand::rngs::OsRng;
use serde_json::json;
use serde_json::Value;
use std::fs;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};

/// Save the keypairs (private and public) to a JSON file.
pub fn save_keys(keypairs: Vec<SigningKey>, file_path: &str) -> io::Result<()> {
    let serialized_keys: Vec<_> = keypairs
        .iter()
        .map(|sk| {
            json!({
                "private_key": general_purpose::STANDARD.encode(sk.to_bytes()),
                "public_key": general_purpose::STANDARD.encode(sk.verifying_key().to_bytes())
            })
        })
        .collect();

    let mut file = fs::File::create(file_path)?;
    writeln!(file, "{}", json!(serialized_keys))?;
    Ok(())
}

/// Load private keys from a JSON file (for the client).
pub fn load_private_keys(file_path: &str) -> Vec<SigningKey> {
    let data = fs::read_to_string(file_path).expect("Failed to read keys file");
    let key_data: Vec<Value> = serde_json::from_str(&data).expect("Failed to parse JSON");
    key_data
        .into_iter()
        .map(|entry| {
            let private_key = general_purpose::STANDARD
                .decode(entry["private_key"].as_str().unwrap())
                .unwrap();
            let key_bytes: [u8; 32] = private_key
                .try_into()
                .expect("Private key must be 32 bytes");
            SigningKey::from_bytes(&key_bytes)
        })
        .collect()
}

/// Load public keys from a JSON file (for the aggregator).
pub fn load_public_keys(file_path: &str) -> Vec<VerifyingKey> {
    let data = fs::read_to_string(file_path).expect("Failed to read keys file");
    let key_data: Vec<Value> = serde_json::from_str(&data).expect("Failed to parse JSON");
    key_data
        .into_iter()
        .map(|entry| {
            let public_key = general_purpose::STANDARD
                .decode(entry["public_key"].as_str().unwrap())
                .unwrap();
            let key_bytes: [u8; 32] = public_key.try_into().expect("Public key must be 32 bytes");
            VerifyingKey::from_bytes(&key_bytes).expect("Invalid public key")
        })
        .collect()
}

pub fn generate_keypairs(num_clients: usize) -> Vec<SigningKey> {
    (0..num_clients)
        .map(|_| SigningKey::generate(&mut OsRng))
        .collect()
}
/// Process WebSocket message to extract price.
pub fn process_message(text: &str) -> Result<f64, Box<dyn std::error::Error>> {
    let json: Value = serde_json::from_str(text)?;
    if let Some(price) = json.get("p") {
        price
            .as_str()
            .to_owned()
            .unwrap_or_default()
            .parse::<f64>()
            .map_err(|e| e.into())
    } else {
        Err("No price field found".into())
    }
}

/// Calculate the average of a vector of numbers.
pub fn calculate_average(prices: &[f64]) -> Option<f64> {
    if prices.is_empty() {
        None
    } else {
        Some(prices.iter().sum::<f64>() / prices.len() as f64)
    }
}

/// Save individual client data to file.
pub fn save_client_data(id: usize, prices: &Vec<f64>, average: f64) -> std::io::Result<()> {
    let mut file = File::create(format!("client_{id}_data.txt"))?;
    writeln!(file, "Prices: {:?}\nAverage: {:.4}", prices, average)?;
    Ok(())
}

pub fn save_client_error_data(id: usize, message: String) -> std::io::Result<()> {
    let mut file = File::create(format!("client_{id}_data.txt"))?;
    writeln!(file, "Error: {}", message)?;
    Ok(())
}

/// Save global aggregator data to file.
pub fn save_global_data(averages: &Vec<f64>, global_average: f64) -> std::io::Result<()> {
    let mut file = File::create("global_data.txt")?;
    writeln!(
        file,
        "Client Averages: {:?}\nGlobal Average: {:.4}",
        averages, global_average
    )?;
    Ok(())
}

/// Parse the command-line arguments
pub fn parse_arguments() -> clap::ArgMatches {
    Command::new("WebSocket Listener")
        .version("1.0")
        .author("Pruthvi Thakor")
        .about("Listens to the WebSocket for BTC/USDT prices")
        .arg(
            Arg::new("mode")
                .short('m')
                .long("mode")
                .value_name("MODE")
                .help("Specifies the mode of operation. Use --mode=cache or --mode=read")
                .required(true),
        )
        .arg(
            Arg::new("times")
                .short('t')
                .long("times")
                .value_name("NUMBER")
                .help("The number of seconds to listen")
                .default_value("1"),
        )
        .get_matches()
}

/// Prints the data after reading it from file
pub fn read_mode(num_clients: usize) -> io::Result<()> {
    println!("Reading prices data ...\n");
    let mut files: Vec<String> = Vec::with_capacity(num_clients + 1);
    for i in 1..=num_clients {
        files.push(format!("client_{}_data.txt", i));
    }
    files.push(String::from("global_data.txt"));
    'file_loop: for file_path in files.iter() {
        // Attempt to open the file
        let file = match File::open(file_path) {
            Ok(file) => file,
            Err(err) => {
                eprintln!("Failed to open {}: {}", file_path, err);
                break 'file_loop; // Exit the loop on error
            }
        };
        println!("\nReading file: {}\n", file_path);
        let reader = BufReader::new(file);

        // Read the file line by line
        for line in reader.lines() {
            match line {
                Ok(content) => println!("{}", content),
                Err(err) => {
                    eprintln!("Error reading a line in {}: {}", file_path, err);
                    break 'file_loop; // Exit the loop on error
                }
            }
        }
    }

    Ok(())
}
