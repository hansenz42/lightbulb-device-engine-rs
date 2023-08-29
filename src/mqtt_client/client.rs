use paho_mqtt as mqtt;
use std::{env, process, time::Duration};

const TOPICS: &[&str] = &["test", "hello"];
const QOS: &[i32] = &[1, 1];

pub fn run() -> std::io::Result<()> {

    let host = env::args()
        .nth(1)
        .unwrap_or_else(|| "mqtt://localhost:1883".to_string());

    // Create the client. Use a Client ID for a persistent session.
    // A real system should try harder to use a unique ID.
    let create_opts = mqtt::CreateOptionsBuilder::new_v3()
        .server_uri(host)
        .client_id("rust_async_subscribe")
        .finalize();


    let mut cli = mqtt::AsyncClient::new(create_opts).unwrap_or_else(|e| {
        println!("Error creating the client: {:?}", e);
        process::exit(1);
    });

    Ok(())
}