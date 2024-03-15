//! 可播放特征

use crate::common::error::DriverError;

pub trait Playable {
    fn play(&mut self, filename: String) -> Result<(), DriverError>;
    fn stop(&mut self, filename: String) -> Result<(), DriverError>;
    fn pause(&self, filename: String) -> Result<(), DriverError>;
    fn resume(&self, filename: String) -> Result<(), DriverError>;
}