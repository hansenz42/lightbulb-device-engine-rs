use crate::entity::dto::device_state_dto::DeviceStateDto;
use crate::{common::error::DriverError, entity::dto::device_command_dto::DeviceCommandDto};
use std::{rc::Rc, sync::mpsc};

/// the device that can send data to upward channel
pub trait ReportUpward {
    fn get_upward_channel(&self) -> &mpsc::Sender<DeviceStateDto>;

    fn report(&self) -> Result<(), DriverError>;

    fn notify_upward(&self, message: DeviceStateDto) -> Result<(), DriverError> {
        let upward_channel = self.get_upward_channel();
        upward_channel.send(message).map_err(|e| {
            DriverError(format!(
                "cannot report to upward device manager, err: {}",
                e
            ))
        })
    }
}

/// the device that operable by controller
pub trait Operable {
    /// send command to operable dvice
    fn operate(&self, message: DeviceCommandDto) -> Result<(), DriverError>;
}

/// root device that controlles device interfaces
pub trait RootBus {
    fn start(&mut self) -> Result<(), DriverError>;
}

/// device that can mount to other device
pub trait SetRef {
    fn set_ref(&mut self, value: Rc<dyn Refable>);
}

/// device that can be mounted by other device
pub trait Refable {}
