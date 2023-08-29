use std::{fs::File, io::Read};
use lazy_static::lazy_static;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Meta {
    pub application_name : String,
    pub scenario_name : String,
    pub server_name: String
}

#[derive(Debug, Deserialize)]
pub struct Env {
    pub env: String,
    pub log_level: String,
    pub debug: bool
}

#[derive(Debug, Deserialize)]
pub struct Server {
    pub server_id: String,
    pub server_ip: String,
    pub server_type: String,
}

#[derive(Debug, Deserialize)]
pub struct Web {
    pub web_host: String,
    pub web_port: i32
}

#[derive(Debug, Deserialize)]
pub struct Pm2 {
    pub process_name: String
}

#[derive(Debug, Deserialize)]
pub struct Mqtt {
    pub broker_host: String,
    pub broker_port: i32,
    pub client_id: String,
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub meta: Meta,
    pub env: Env,
    pub server: Server,
    pub web: Web,
    pub pm2: Pm2,
    pub mqtt: Mqtt
}

impl Default for Settings {
    fn default() -> Self {
        let file_path = "config.toml";
        let mut file = match File::open(file_path) {
            Ok(f) => f,
            Err(e) => panic!("no such file {} exception:{}", file_path, e)
        };
        let mut str_val = String::new();
        match file.read_to_string(&mut str_val) {
            Ok(s) => s,
            Err(e) => panic!("Error Reading file: {}", e)
        };
        toml::from_str(&str_val).expect("Parsing the configuration file failed")
    }
}

impl Settings {
    pub fn get<'a>() -> &'a Self {
        // 给静态变量延迟赋值的宏
        lazy_static! {
            static ref CACHE: Settings = Settings::default();
        }
        &CACHE
    }
}