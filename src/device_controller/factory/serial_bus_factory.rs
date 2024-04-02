use serde_json::Value;
use crate::{common::error::DriverError, driver::serial::serial_bus::SerialBus };
use crate::util::json;

const DEVICE_IDENTIFIER: &str = "serial_bus";


pub fn make(json_data: &Value) -> Result<SerialBus, DriverError> {
    let _ = json::check_type(json_data, DEVICE_IDENTIFIER)?;

    let device_id = json::get_str(json_data, "device_id")?;
    let serial_port = json::get_config_str(json_data, "serial_port")?;
    let baudrate = json::get_config_int(json_data, "baudrate")?;
    let obj = SerialBus::new(
        device_id.as_str(), 
        serial_port.as_str(), 
        baudrate.try_into().map_err(
            |e| DriverError(format!("device factory: cannot convert baudrate to int, err: {e}"))
        )?
    ); 
    Ok(obj)
}