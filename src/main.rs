mod http_server;
mod mqtt_client;
mod common;
mod driver;
mod device_controller;
mod file_controller;
mod entity;
mod util;
use ctrlc;
use http_server::server::CustomHttpServer;
use common::setting::Settings;
use common::logger::init_logger;
use common::sqlite::SqliteConnection;
use std::error::Error;
// use crate::device_controller::device_manager::DeviceManager;
use crate::file_controller::file_manager::FileManager;
use log;
use dotenv::dotenv;
use mqtt_client::client::MqttClient;
use tokio;

// #[macro_use] extern crate log;

const LOG_TAG: &str = "main";

fn main() {}

// fn main() -> Result<(), Box<dyn Error>> {
//     // 检查 env 文件
//     dotenv().ok();

//     // 加载 config
//     let settings = Settings::get();

//     // 设置 logger
//     init_logger()?;
//     info!(LOG_TAG, "配置已加载，环境: {:?}", settings.env.env);
//     debug!(LOG_TAG, "配置: {:?}", settings);

//     let rt = tokio::runtime::Runtime::new().unwrap();

//     rt.block_on(async move {
//         // 设备管理器
//         let mut device_manager = DeviceManager::new();
//         device_manager.startup().await.expect("设备管理器启动失败");

//         // 文件管理器 fileManager
//         let mut file_manager = FileManager::new();
//         file_manager.startup().await.expect("文件管理器启动失败");

//         // 执行 http 服务器
//         CustomHttpServer::start().await.unwrap();

//         // 执行 mqtt 服务器
//         let mut client = MqttClient::new();
//         client.start().await.unwrap();
//     });

//     ctrlc::set_handler(move || {
//         info!(LOG_TAG, "收到退出信号，退出进程");
//         std::process::exit(0);
//     });

//     info!(LOG_TAG, "deviceserver 启动完成，进入循环");
//     // loop 主线程
//     loop {}

//     Ok(())
// }

// fn playground() {

// }