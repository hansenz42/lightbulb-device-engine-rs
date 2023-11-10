use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fmt;

#[derive(Debug, Copy, Clone)]
pub enum ErrorCode {
    // 未知错误
    UnknownError = 1000,
    // http 请求错误
    HttpError = 1001,
    // 设备配置文件错误
    DeviceConfigError = 1002,
    // 设备配置错误
    DeviceTypeNotSupport = 1003,
}

#[derive(Debug)]
pub struct DeviceServerError {
    pub code: ErrorCode,
    pub msg: String,
}

impl Display for DeviceServerError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "设备服务器错误 code: {},  msg: {}", self.code as u8, self.msg)
    }
}

impl Error for DeviceServerError {}

// 设备驱动错误
#[derive(Debug)]
pub struct DriverError(pub String);

impl Display for DriverError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "设备驱动错误 msg: {}", self.0)
    }
}

impl Error for DriverError {}