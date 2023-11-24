//! 可播放特征

use crate::common::error::DriverError;

pub trait Playable {
    fn play(&self, filename: String) -> Result<(), DriverError>;
    fn stop(&self) -> Result<(), DriverError>;
    fn pause(&self) -> Result<(), DriverError>;
    fn resume(&self) -> Result<(), DriverError>;
}