use crate::entity::dto::device_state_dto::StateToDeviceControllerDto;
use crate::{common::error::DriverError, entity::dto::device_command_dto::DeviceCommandDto};
use std::{rc::Rc, sync::mpsc};

/// the device that can send data to upward channel
pub trait ReportUpward {
    fn get_upward_channel(&self) -> &mpsc::Sender<StateToDeviceControllerDto>;

    fn report(&self) -> Result<(), DriverError>;

    fn notify_upward(&self, message: StateToDeviceControllerDto) -> Result<(), DriverError> {
        let upward_channel = self.get_upward_channel();
        upward_channel.send(message).map_err(|e| {
            DriverError(format!(
                "cannot report to upward device manager, err: {}",
                e
            ))
        })
    }
}

/// the device that can be commanded by device manager
pub trait Commandable {
    fn cmd(&mut self, dto: DeviceCommandDto) -> Result<(), DriverError>;
}

/// device that can be mounted by other device
pub trait Refable {}
