use crate::entity::dto::device_info_dto::DeviceMetaInfoDto;
use crate::{common::error::DriverError,  driver::modbus::modbus_di_controller::ModbusDiController};
use crate::util::json;

pub fn make( device_info: &DeviceMetaInfoDto) -> Result<ModbusDiController, DriverError> {
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