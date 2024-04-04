use std::{borrow::Borrow, cell::RefCell, rc::Rc};

use serde_json::Value;
use crate::{common::error::DriverError, device_controller::entity::device_enum::DeviceRefEnum, driver::modbus::{modbus_bus::ModbusBus, modbus_do_controller::ModbusDoController}};
use crate::util::json;

const DEVICE_IDENTIFIER: &str = "modbus_do_controller";

pub fn make(json_data: &Value, modbus_ref: &Rc<RefCell<ModbusBus>>) -> Result<ModbusDoController, DriverError> {
    let _ = json::check_type(json_data, DEVICE_IDENTIFIER)?;

    let device_id = json::get_str(json_data, "device_id")?;
    let unit = json::get_config_int(json_data, "unit")?;
    let output_num = json::get_config_int(json_data, "output_num")?;
    let obj = ModbusDoController::new(
        device_id.as_str(), 
        unit.try_into().map_err(
            |e| DriverError(format!("device factory: cannot convert unit to int, err: {e}"))
        )?, 
        output_num.try_into().map_err(
            |e| DriverError(format!("device factory: cannot convert output_num to int, err: {e}"))
        )?, 
        Rc::clone(modbus_ref)
    ); 
    Ok(obj)
}