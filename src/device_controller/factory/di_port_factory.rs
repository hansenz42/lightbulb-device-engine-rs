use std::sync::mpsc;

use serde_json::Value;
use crate::{common::error::DriverError, device_controller::entity::device_info::DeviceInfoDto, driver::modbus::{modbus_bus::ModbusBus, modbus_di_port::ModbusDiPort}, entity::dto::device_state_dto::DeviceStateDto};
use crate::util::json;

const DEVICE_IDENTIFIER: &str = "modbus_di_port";

pub fn make(device_info: &DeviceInfoDto, upward_channel: mpsc::Sender<DeviceStateDto>) -> Result<ModbusDiPort, DriverError> {
    let address = json::get_config_int(&device_info.config, "address")?;
    let obj = ModbusDiPort::new(
        device_info.device_id.as_str(),
        address.try_into().map_err(
            |e| DriverError(format!("device factory: cannot convert address to int, err: {e}"))
        )?, 
        upward_channel
    ); 
    Ok(obj)
}