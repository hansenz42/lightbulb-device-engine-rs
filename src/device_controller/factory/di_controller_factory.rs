use crate::{common::error::DriverError, device_controller::entity::device_info::DeviceInfoDto, driver::modbus::{modbus_bus::ModbusBus, modbus_di_controller::ModbusDiController, modbus_do_controller::ModbusDoController}};
use crate::util::json;

const DEVICE_IDENTIFIER: &str = "modbus_di_controller";

pub fn make( device_info: &DeviceInfoDto) -> Result<ModbusDiController, DriverError> {
    let unit = json::get_config_int(&device_info.config, "unit")?;
    let input_num = json::get_config_int(&device_info.config, "input_num")?;
    let obj = ModbusDiController::new(
        device_info.device_id.as_str(), 
        unit.try_into().map_err(
            |e| DriverError(format!("device factory: cannot convert unit to int, err: {e}"))
        )?, 
        input_num.try_into().map_err(
            |e| DriverError(format!("device factory: cannot convert input_num to int, err: {e}"))
        )?
    ); 
    Ok(obj)
}