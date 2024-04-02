use serde_json::Value;
use crate::{common::error::DriverError, driver::dmx::dmx_bus::DmxBus};
use crate::util::json;

const DEVICE_IDENTIFIER: &str = "dmx_bus";

pub fn make(json_data: &Value) -> Result<DmxBus, DriverError> {
    let _ = json::check_type(json_data, DEVICE_IDENTIFIER);

    let device_id = json::get_str(json_data, "device_id")?;
    let serial_port = json::get_str(json_data, "serial_port")?;
    let obj = DmxBus::new(
        device_id.as_str(),
        serial_port.as_str()
    ); 
    Ok(obj)
}