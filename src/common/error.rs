use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fmt;

#[derive(Debug, Copy, Clone)]
pub enum ServerErrorCode {
    UnknownError = 1000,
    HttpError = 1001,
    // Error related to device config files
    DeviceInfoError = 1002,
    // the device type is not supportted
    DeviceTypeNotSupport = 1003,
    // cannot read or write files
    FileSystemError = 1004,
    DatabaseError = 1005,
    // error in processing file config
    FileConfigError = 1006,
    // error in mqtt Connect
    MqttError = 1007,
}

#[derive(Debug)]
pub struct DeviceServerError {
    pub code: ServerErrorCode,
    pub msg: String,
}

impl Display for DeviceServerError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "device server error, code: {},  msg: {}", self.code as u8, self.msg)
    }
}

impl Error for DeviceServerError {}

// error for drivers
#[derive(Debug)]
pub struct DriverError(pub String);

impl Display for DriverError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "device driver error msg: {}", self.0)
    }
}

impl Error for DriverError {}