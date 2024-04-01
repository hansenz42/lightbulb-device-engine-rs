use serde_json::Value;
use crate::{common::error::DriverError, driver::modbus::{modbus_do_controller::ModbusDoController, modbus_do_port::ModbusDoPort}};
use super::util;

const DEVICE_IDENTIFIER: &str = "modbus_do_port";

pub fn make<'a>(json_data: &Value, modbus_do_controller_ref: &'a ModbusDoController) -> Result<ModbusDoPort<'a>, DriverError> {
    let _ = util::check_type(json_data, DEVICE_IDENTIFIER)?;

    let device_id = util::get_str(json_data, "device_id")?;
    let address = util::get_config_int(json_data, "address")?;
    let obj = ModbusDoPort::new(
        device_id, 
        address.try_into().map_err(
            |e| DriverError(format!("device factory: cannot convert address to int, err: {e}"))
        )?, 
        modbus_do_controller_ref
    ); 
    Ok(obj)
}