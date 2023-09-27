//! 日志配置模块
//! 用于管理日志输出

use std::io::Error;

use pretty_env_logger;

pub fn init_logger() -> Result<(), Error> {
    pretty_env_logger::formatted_builder()
        .target(pretty_env_logger::env_logger::Target::Stdout)
        .filter_level(log::LevelFilter::Debug)
        .init();
    Ok(())
}