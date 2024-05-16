use std::sync::mpsc::Sender;

use serde_json::Value;
use crate::entity::dto::device_meta_info_dto::DeviceMetaInfoDto;
use crate::entity::dto::device_state_dto::DeviceStateDto;
use crate::{common::error::DriverError, driver::modbus::modbus_bus::ModbusBus };
use crate::util::json;

pub fn make(device_info: &DeviceMetaInfoDto, report_tx: Sender<DeviceStateDto>) -> Result<ModbusBus, DriverError> {
    let serial_port = json::get_config_str(&device_info.config, "serial_port")?;
    let baudrate = json::get_config_int(&device_info.config, "baudrate")?;
    let obj = ModbusBus::new(
        &device_info.device_id,
        serial_port.as_str(), 
        baudrate.try_into().map_err(
        |e| DriverError(format!("device factory: cannot convert baudrate to int, err: {e}"))
        )?,
        report_tx
    ); 
    Ok(obj)
}
