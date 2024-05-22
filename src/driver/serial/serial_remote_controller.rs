use std::sync::mpsc::Sender;
use rodio::Device;

use crate::driver::traits::ReportUpward;
use crate::entity::dto::device_report_dto::DeviceReportDto;
use crate::entity::dto::device_state_dto::{DeviceStateDto, RemoteStateDto, StateDtoEnum};
use crate::common::error::DriverError;
use super::traits::SerialMountable;
use std::env;
use crate::{info, warn, error, trace, debug};
use super::entity::SerialDataBo;

const LOG_TAG: &str = "serial_remote_controller";
const DEVICE_CLASS: &str = "operable";
const DEVICE_TYPE: &str = "remote";

pub struct SerialRemoteController {
    device_id: String,
    button_num: u8,
    report_channel: Sender<DeviceStateDto>,
    error_msg: Option<String>,
    error_timestamp: Option<u64>,
    last_update: Option<u64>,
}


impl SerialRemoteController {
    pub fn new(device_id: &str, button_num: u8, report_channel: Sender<DeviceStateDto>) -> Self {
        SerialRemoteController {
            device_id: device_id.to_string(),
            button_num,
            report_channel,
            error_msg: None,
            error_timestamp: None,
            last_update: None,
        }
    }
}

impl SerialMountable for SerialRemoteController {
    fn notify(&self, data: SerialDataBo) -> Result<(), DriverError> {
        let env_mode = env::var("mode").unwrap_or("real".to_string());
        if env_mode == "dummy" {
            info!(LOG_TAG, "serial remote controller is in dummy mode, receive remote data: {:?}", &data.data);
        } else {
            if let Some(pressed_num) = data.data.get(0) {
                let state = StateDtoEnum::Remote(RemoteStateDto{
                    pressed: pressed_num.clone(),
                });
                let dto = DeviceStateDto {
                    device_id: self.device_id.clone(),
                    device_class: DEVICE_CLASS.to_string(),
                    device_type: DEVICE_TYPE.to_string(),
                    status: DeviceReportDto{
                        active: true,
                        error_msg: self.error_msg.clone(),
                        error_timestamp: self.error_timestamp.clone(),
                        last_update: self.last_update.clone(),
                        state: state
                    },
                };
                let _ = self.notify_upward(dto)?;
                debug!(LOG_TAG, "the remote controller pressed, number: {}", pressed_num);
            }
        }
        Ok(())
    }
}

impl ReportUpward for SerialRemoteController {
    fn get_upward_channel(&self) -> &Sender<DeviceStateDto> {
        return &self.report_channel;
    }

    fn report(&self) -> Result<(), DriverError> {
        Ok(())
    }
}