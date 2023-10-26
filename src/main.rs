mod http_server;
mod mqtt_client;
mod common;
mod driver;
mod device_controller;
mod file_controller;
mod entity;
mod util;
use file_controller::file_manager::FileManager;
use http_server::server::CustomHttpServer;
use common::setting::Settings;
use common::logger::init_logger;
use common::sqlite::SqliteConnection;
use std::error::Error;
use log;
use dotenv::dotenv;
use mqtt_client::client::MqttClient;
use tokio;

fn main() -> Result<(), Box<dyn Error>> {
    // 检查 env 文件
    dotenv().ok();

    // 加载 config
    let settings = Settings::get();

    // 设置 logger
    init_logger()?;
    log::info!("配置已加载，环境: {:?}", settings.env.env);
    log::debug!("配置: {:?}", settings);

    let rt = tokio::runtime::Runtime::new().unwrap();

    rt.block_on(async move {
        // 初始化设备管理器

        // 初始化文件管理器 fileManager
        

        // 执行 http 服务器
        CustomHttpServer::start().await.unwrap();

        // 执行 mqtt 服务器
        let mut client = MqttClient::new();
        client.start().await.unwrap();
    });

    // loop 主线程
    loop {}

    Ok(())
}

fn playground() {

}