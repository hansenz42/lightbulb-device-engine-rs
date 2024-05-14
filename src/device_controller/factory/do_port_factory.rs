use std::{cell::RefCell, rc::Rc};

use serde_json::Value;
use crate::{common::error::DriverError, driver::modbus::{modbus_do_controller::ModbusDoController, modbus_do_port::ModbusDoPort}, entity::dto::device_info_dto::DeviceMetaInfoDto, util::json};

pub fn make(device_info: &DeviceMetaInfoDto, modbus_do_controller_ref: Rc<RefCell<ModbusDoController>>) -> Result<ModbusDoPort, DriverError> {
    let address = json::get_config_int(&device_info.config, "address")?;
    let obj = ModbusDoPort::new(
        device_info.device_id.as_str(), 
        address.try_into().map_err(
            |e| DriverError(format!("device factory: cannot convert address to int, err: {e}"))
        )?, 
        modbus_do_controller_ref
    ); 
    Ok(obj)
}