use serde_json::Value;
use crate::device_controller::entity::device_info::DeviceInfoDto;
use crate::{common::error::DriverError, driver::dmx::dmx_bus::DmxBus};
use crate::util::json;

const DEVICE_IDENTIFIER: &str = "dmx_bus";

pub fn make(device_info: &DeviceInfoDto) -> Result<DmxBus, DriverError> {
    let serial_port = json::get_str(&device_info.config, "serial_port")?;
    let obj = DmxBus::new(
        device_info.device_id.as_str(),
        serial_port.as_str()
    ); 
    Ok(obj)
}