use serde_json::Value;
use crate::{common::error::DriverError, driver::dmx::dmx_bus::DmxBus};

struct DmxBusFactory;

pub fn make_dmx_bus(value: Value) -> Result<DmxBus, DriverError> {
    let device_type = value["device_type"].as_str().ok_or(DriverError("dmx factory: cannot find device_type in config".to_string()))?;
    if device_type != "dmx_bus" {
        return Err(DriverError("dmx factory: device_type must be dmx_bus".to_string()));
    }

    let device_id = value["device_id"].as_str().ok_or(DriverError("dmx factory: cannot find device_id in config".to_string()))?;
    let serial_port = value["serial_port"].as_str().ok_or(DriverError("dmx factory: cannot find serial_port in config".to_string()))?;
    let obj = DmxBus::new(device_id, serial_port); 
    Ok(obj)
}