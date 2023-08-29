//! 日志配置模块
//! 用于管理日志输出

use std::io::Error;

use pretty_env_logger;

pub fn init_logger() -> Result<(), Error> {
    pretty_env_logger::init();
    Ok(())
}