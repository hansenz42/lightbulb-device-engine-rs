use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc::Sender;

use super::dmx_bus::DmxBus;
use super::prelude::*;
use super::traits::DmxCaller;
use crate::common::error::DriverError;
use crate::driver::traits::ReportUpward;
use crate::entity::dto::device_report_dto::DeviceReportDto;
use crate::entity::dto::device_state_dto::{
    ChannelStateDto, DmxBusStateDto, StateDtoEnum, StateToDeviceControllerDto,
};

const DEVICE_CLASS: &str = "operable";
const DEVICE_TYPE: &str = "dmx_channel";

/// channelled device that send data to modbus
pub struct DmxChannelDevice {
    device_id: String,
    address: DmxAddress,
    channel_num: DmxAddress,
    value: Vec<DmxValue>,
    dmx_bus_ref: Rc<RefCell<DmxBus>>,
    report_tx: Sender<StateToDeviceControllerDto>,
    error_msg: Option<String>,
    error_timestamp: Option<u64>,
    last_update: Option<u64>,
}

impl DmxCaller for DmxChannelDevice {
    fn set_channel(&mut self, channel: DmxAddress, value: DmxValue) -> Result<(), DriverError> {
        // check if the channel is out of range
        if channel > self.channel_num {
            return Err(DriverError(format!("channelled device set_channel failed, channel out of range, channel = {}, channel_num = {}, device_id = {}", channel, self.channel_num, self.device_id)));
        }
        // update data in vec
        self.value[channel as usize] = value;
        self.dmx_bus_ref
            .borrow_mut()
            .set_channel(self.address + channel, value)?;
        Ok(())
    }
}

impl DmxChannelDevice {
    pub fn new(
        device_id: &str,
        address: DmxAddress,
        channel_num: DmxAddress,
        dmx_bus: Rc<RefCell<DmxBus>>,
        report_tx: Sender<StateToDeviceControllerDto>,
    ) -> Self {
        DmxChannelDevice {
            device_id: device_id.to_string(),
            address,
            channel_num,
            value: vec![0; channel_num as usize],
            dmx_bus_ref: dmx_bus,
            report_tx,
            error_msg: None,
            error_timestamp: None,
            last_update: None,
        }
    }
}

impl ReportUpward for DmxChannelDevice {
    fn get_upward_channel(&self) -> &std::sync::mpsc::Sender<StateToDeviceControllerDto> {
        &self.report_tx
    }

    fn report(&self) -> Result<(), DriverError> {
        let state_dto = ChannelStateDto {
            address: self.address,
            channels: self.value.clone(),
        };
        self.notify_upward(StateToDeviceControllerDto {
            device_id: self.device_id.clone(),
            device_class: DEVICE_CLASS.to_string(),
            device_type: DEVICE_TYPE.to_string(),
            status: DeviceReportDto {
                error_msg: self.error_msg.clone(),
                error_timestamp: self.error_timestamp,
                last_update: self.last_update,
                state: StateDtoEnum::Channel(state_dto),
                active: true,
            },
        })?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::common::logger::init_logger;
    use std::env;
    use std::thread;

    fn set_env() {
        env::set_var("dummy", "true");
        let _ = init_logger();
    }

    /// test send data to dmx bus
    #[test]
    fn test_channel_device() {
        set_env();

        // let mut dmx_bus = DmxBus::new("test", "test");
        // let _ = dmx_bus.start();
        // let mut dmx_channel_device = DmxChannelDevice::new("test", 1, 255, &mut dmx_bus);
        // let _ = dmx_channel_device.set_channel(3, 255);

        thread::sleep(std::time::Duration::from_secs(60));
    }
}
