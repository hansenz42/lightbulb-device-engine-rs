use serde_json::Value;
use crate::{common::error::DriverError, driver::modbus::{modbus_bus::ModbusBus, modbus_do_controller::ModbusDoController}};
use super::util;

const DEVICE_IDENTIFIER: &str = "modbus_do_controller";

pub fn make<'a>( json_data: &Value, modbus_ref: &'a ModbusBus) -> Result<ModbusDoController<'a>, DriverError> {
    let _ = util::check_type(json_data, DEVICE_IDENTIFIER)?;

    let device_id = util::get_str(json_data, "device_id")?;
    let unit = util::get_config_int(json_data, "unit")?;
    let output_num = util::get_config_int(json_data, "output_num")?;
    let obj = ModbusDoController::new(
        device_id, 
        unit.try_into().map_err(
            |e| DriverError(format!("device factory: cannot convert unit to int, err: {e}"))
        )?, 
        output_num.try_into().map_err(
            |e| DriverError(format!("device factory: cannot convert output_num to int, err: {e}"))
        )?, 
        modbus_ref
    ); 
    Ok(obj)
}