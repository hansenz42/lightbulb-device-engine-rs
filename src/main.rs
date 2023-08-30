mod http_server;
mod mqtt_client;
mod common;
use http_server::server::run as http_run;
use mqtt_client::client::run as mqtt_run;
use common::setting::Settings;
use common::logger::init_logger;
use std::error::Error;
use log;
use dotenv::dotenv;


fn main() -> Result<(), Box<dyn Error>> {
    // 检查 env 文件
    dotenv().ok();

    // 加载 config
    let settings = Settings::get();

    // 设置 logger
    init_logger()?;
    log::info!("配置已加载，环境: {:?}", settings.env.env);
    log::debug!("配置: {:?}", settings);

    // 执行 http 服务器
    // http_run()?;

    Ok(())
}