use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fmt;

#[derive(Debug, Copy, Clone)]
pub enum ErrorCode {
    // 未知错误
    UnknownError = 3000,
    // http 请求错误
    HttpError = 3001,
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