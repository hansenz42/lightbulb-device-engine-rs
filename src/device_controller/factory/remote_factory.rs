
use std::sync::mpsc::Sender;

use serde_json::Value;
use crate::driver::serial::serial_remote_controller::SerialRemoteController;
use crate::entity::dto::device_meta_info_dto::DeviceMetaInfoDto;
use crate::entity::dto::device_state_dto::DeviceStateDto;
use crate::{common::error::DriverError };
use crate::util::json;


pub fn make(device_info: &DeviceMetaInfoDto, report_tx: Sender<DeviceStateDto>) -> Result<SerialRemoteController, DriverError> {
    let button_num_i64 = json::get_config_int(&device_info.config, "button_num")?;
    let button_num: u8 = button_num_i64.try_into()
        .map_err(|_| DriverError("init remote controller error, cannot transform button_num to u8".to_string()))?;
    
    let obj = SerialRemoteController::new(
        device_info.device_id.as_str(), 
        button_num,
        report_tx
    ); 
    Ok(obj)
}