use crate::common::error::DriverError;
use super::entity::SerialDataBo;

/// device that can be mounted to serial bus
pub trait SerialMountable {
    /// notify the device, then the device will call upward channel
    fn notify(&self, data: SerialDataBo) -> Result<(), DriverError>;
}