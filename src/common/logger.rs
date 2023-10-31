//! 日志配置模块
//! 用于管理日志输出

use std::io::Error;

use pretty_env_logger;

/// 初始化日志的基础功能
pub fn init_logger() -> Result<(), Error> {
    pretty_env_logger::formatted_builder()
        .target(pretty_env_logger::env_logger::Target::Stdout)
        .filter_level(log::LevelFilter::Debug)
        .init();
    Ok(())
}


/// 带 TAG 输出到日志的宏，支持 trace debug info warn error
#[macro_export]
macro_rules! warn {
    ($tag:expr, $($arg:tt)*) => ({
        log::warn!("[{}] {}", $tag, format_args!($($arg)*));
    })
}

#[macro_export]
macro_rules! error {
    ($tag:expr, $($arg:tt)*) => ({
        log::error!("[{}] {}", $tag, format_args!($($arg)*));
    })
}

#[macro_export]
macro_rules! info {
    ($tag:expr, $($arg:tt)*) => ({
        log::info!("[{}] {}", $tag, format_args!($($arg)*));
    })
}

#[macro_export]
macro_rules! debug {
    ($tag:expr, $($arg:tt)*) => ({
        log::debug!("[{}] {}", $tag, format_args!($($arg)*));
    })
}

#[macro_export]
macro_rules! trace {
    ($tag:expr, $($arg:tt)*) => ({
        log::trace!("[{}] {}", $tag, format_args!($($arg)*));
    })
}