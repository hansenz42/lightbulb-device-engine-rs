use std::{cell::RefCell, rc::Rc};

use serde_json::Value;
use crate::{common::error::DriverError, device_controller::entity::device_info::DeviceInfoDto, driver::modbus::{modbus_do_controller::ModbusDoController, modbus_do_port::ModbusDoPort}};
use crate::util::json;

const DEVICE_IDENTIFIER: &str = "modbus_do_port";

pub fn make(device_info: &DeviceInfoDto, modbus_do_controller_ref: Rc<RefCell<ModbusDoController>>) -> Result<ModbusDoPort, DriverError> {
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