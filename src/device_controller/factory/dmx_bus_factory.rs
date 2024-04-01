use serde_json::Value;
use crate::{common::error::DriverError, driver::dmx::dmx_bus::DmxBus};
use super::util;

const DEVICE_IDENTIFIER: &str = "dmx_bus";

pub fn make(json_data: &Value) -> Result<DmxBus, DriverError> {
    let _ = util::check_type(json_data, DEVICE_IDENTIFIER);

    let device_id = util::get_str(json_data, "device_id")?;
    let serial_port = util::get_str(json_data, "serial_port")?;
    let obj = DmxBus::new(device_id, serial_port); 
    Ok(obj)
}