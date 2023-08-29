use paho_mqtt as mqtt;
use std::{process, time::Duration};
use crate::common::setting::Settings;

const TOPICS: &[&str] = &["test", "hello"];
const QOS: &[i32] = &[1, 1];


pub fn run() -> std::io::Result<()> {
    let host = format!("mqtt://{}:{}", Settings::get().mqtt.broker_host, Settings::get().mqtt.broker_port);

    let create_opts = mqtt::CreateOptionsBuilder::new_v3()
        .server_uri(host)
        .client_id(Settings::get().mqtt.client_id.as_str())
        .finalize();


    let mut cli = mqtt::AsyncClient::new(create_opts).unwrap_or_else(|e| {
        println!("Error creating the client: {:?}", e);
        process::exit(1);
    });

    Ok(())
}