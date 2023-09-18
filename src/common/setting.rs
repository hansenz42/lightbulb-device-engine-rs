//! 配置文件管理器

use std::{fs::File, io::Read};
use lazy_static::lazy_static;
use serde::Deserialize;
use std::env;

#[derive(Debug, Deserialize)]
pub struct Meta {
    pub application_name : String,
    pub scenario_name : String,
    pub server_name: String
}

#[derive(Debug, Deserialize)]
pub struct Env {
    pub debug: bool,
    pub env: String
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
    pub web_port: u16
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
pub struct Upstream {
    pub host: String,
    pub port: u16
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub meta: Meta,
    pub env: Env,
    pub server: Server,
    pub web: Web,
    pub pm2: Pm2,
    pub mqtt: Mqtt,
    pub upstream: Upstream
}

impl Default for Settings {
    fn default() -> Self {
        let env = env::var("ENV").expect("环境变量 env 未设置，请检查环境变量配置或 .env 文件是否存在");
        let file_path: String = format!("config_{}.toml", env);

        let mut file = match File::open(file_path.as_str()) {
            Ok(f) => f,
            Err(e) => panic!("no such file {} exception:{}", file_path.as_str(), e)
        };

        let mut str_val = String::new();

        match file.read_to_string(&mut str_val) {
            Ok(s) => s,
            Err(e) => panic!("加载配置文件失败: {}", e)
        };

        toml::from_str(&str_val).expect("配置文件格式不正确")
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