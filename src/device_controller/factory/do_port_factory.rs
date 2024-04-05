use std::{cell::RefCell, rc::Rc};

use serde_json::Value;
use crate::{common::error::DriverError, driver::modbus::{modbus_do_controller::ModbusDoController, modbus_do_port::ModbusDoPort}};
use crate::util::json;

const DEVICE_IDENTIFIER: &str = "modbus_do_port";

pub fn make(json_data: &Value, modbus_do_controller_ref: Rc<RefCell<ModbusDoController>>) -> Result<ModbusDoPort, DriverError> {
    let _ = json::check_type(json_data, DEVICE_IDENTIFIER)?;

    let device_id = json::get_str(json_data, "device_id")?;
    let address = json::get_config_int(json_data, "address")?;
    let obj = ModbusDoPort::new(
        device_id.as_str(), 
        address.try_into().map_err(
            |e| DriverError(format!("device factory: cannot convert address to int, err: {e}"))
        )?, 
        modbus_do_controller_ref
    ); 
    Ok(obj)
}