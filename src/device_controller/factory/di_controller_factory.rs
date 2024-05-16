use std::sync::mpsc::Sender;

use crate::entity::dto::device_meta_info_dto::DeviceMetaInfoDto;
use crate::entity::dto::device_state_dto::DeviceStateDto;
use crate::{common::error::DriverError,  driver::modbus::modbus_di_controller::ModbusDiController};
use crate::util::json;

pub fn make( device_info: &DeviceMetaInfoDto, report_tx: Sender<DeviceStateDto>) -> Result<ModbusDiController, DriverError> {
    let unit = json::get_config_int(&device_info.config, "unit")?;
    let input_num = json::get_config_int(&device_info.config, "num")?;
    let obj = ModbusDiController::new(
        device_info.device_id.as_str(), 
        unit.try_into().map_err(
            |e| DriverError(format!("device factory: cannot convert unit to int, err: {e}"))
        )?, 
        input_num.try_into().map_err(
            |e| DriverError(format!("device factory: cannot convert num to int, err: {e}"))
        )?,
        report_tx
    ); 
    Ok(obj)
}