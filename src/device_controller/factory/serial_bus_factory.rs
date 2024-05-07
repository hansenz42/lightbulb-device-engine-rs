use serde_json::Value;
use crate::device_controller::entity::device_info::DeviceInfoDto;
use crate::{common::error::DriverError, driver::serial::serial_bus::SerialBus };
use crate::util::json;

const DEVICE_IDENTIFIER: &str = "serial_bus";


pub fn make(device_info: &DeviceInfoDto) -> Result<SerialBus, DriverError> {
    let serial_port = json::get_config_str(&device_info.config, "serial_port")?;
    let baudrate = json::get_config_int(&device_info.config, "baudrate")?;
    let obj = SerialBus::new(
        device_info.device_id.as_str(), 
        serial_port.as_str(), 
        baudrate.try_into().map_err(
            |e| DriverError(format!("device factory: cannot convert baudrate to int, err: {e}"))
        )?
    ); 
    Ok(obj)
}