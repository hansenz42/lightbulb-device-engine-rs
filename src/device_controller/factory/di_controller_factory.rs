use serde_json::Value;
use crate::{common::error::DriverError, driver::modbus::{modbus_bus::ModbusBus, modbus_di_controller::ModbusDiController, modbus_do_controller::ModbusDoController}};
use crate::util::json;

const DEVICE_IDENTIFIER: &str = "modbus_di_controller";

pub fn make( json_data: &Value) -> Result<ModbusDiController, DriverError> {
    let _ = json::check_type(json_data, DEVICE_IDENTIFIER)?;

    let device_id = json::get_str(json_data, "device_id")?;
    let unit = json::get_config_int(json_data, "unit")?;
    let input_num = json::get_config_int(json_data, "input_num")?;
    let obj = ModbusDiController::new(
        device_id.as_str(), 
        unit.try_into().map_err(
            |e| DriverError(format!("device factory: cannot convert unit to int, err: {e}"))
        )?, 
        input_num.try_into().map_err(
            |e| DriverError(format!("device factory: cannot convert input_num to int, err: {e}"))
        )?
    ); 
    Ok(obj)
}