use serde_json::Value;
use crate::device_controller::entity::device_info::DeviceInfoDto;
use crate::{common::error::DriverError, driver::modbus::modbus_bus::ModbusBus };
use crate::util::json;

const DEVICE_IDENTIFIER: &str = "modbus_bus";

pub fn make(device_info: &DeviceInfoDto) -> Result<ModbusBus, DriverError> {
    if device_info.device_type != DEVICE_IDENTIFIER {
        return Err(DriverError("device factory: device type is not modbus_bus".to_string()));
    }

    let serial_port = json::get_config_str(&device_info.config, "serial_port")?;
    let baudrate = json::get_config_int(&device_info.config, "baudrate")?;
    let obj = ModbusBus::new(
        &device_info.device_id,
        serial_port.as_str(), 
        baudrate.try_into().map_err(
        |e| DriverError(format!("device factory: cannot convert baudrate to int, err: {e}"))
    )?); 
    Ok(obj)
}
