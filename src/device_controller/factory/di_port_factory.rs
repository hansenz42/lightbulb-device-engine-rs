use std::sync::mpsc;

use serde_json::Value;
use crate::{common::error::DriverError, driver::modbus::{modbus_bus::ModbusBus, modbus_di_port::ModbusDiPort}, entity::bo::device_state_bo::DeviceStateBo};
use crate::util::json;

const DEVICE_IDENTIFIER: &str = "modbus_di_port";

pub fn make(json_data: &Value, upward_channel: mpsc::Sender<DeviceStateBo>) -> Result<ModbusDiPort, DriverError> {
    let _ = json::check_type(json_data, DEVICE_IDENTIFIER)?;

    let device_id = json::get_str(json_data, "device_id")?;
    let address = json::get_config_int(json_data, "address")?;
    let obj = ModbusDiPort::new(
        device_id.as_str(),
        address.try_into().map_err(
            |e| DriverError(format!("device factory: cannot convert address to int, err: {e}"))
        )?, 
        upward_channel
    ); 
    Ok(obj)
}