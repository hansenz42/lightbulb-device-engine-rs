use std::sync::mpsc::Sender;
use crate::driver::traits::UpwardSendable;
use crate::entity::dto::device_state_dto::{DeviceStateDto, RemoteStateDto, StateDtoEnum};
use crate::common::error::DriverError;
use super::traits::SerialMountable;
use std::env;
use crate::{info, warn, error, trace, debug};
use super::entity::SerialDataBo;

const LOG_TAG: &str = "serial_remote_controller.rs | serial remote controller";
const DEVICE_CLASS: &str = "operable";
const DEVICE_TYPE: &str = "remote";

pub struct SerialRemoteController {
    device_id: String,
    button_num: u8,
    upward_channel: Sender<DeviceStateDto>
}


impl SerialRemoteController {
    pub fn new(device_id: &str, button_num: u8, upward_channel: Sender<DeviceStateDto>) -> Self {
        SerialRemoteController {
            device_id: device_id.to_string(),
            button_num,
            upward_channel
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
                let device_state_bo = DeviceStateDto {
                    device_id: self.device_id.clone(),
                    device_class: DEVICE_CLASS.to_string(),
                    device_type: DEVICE_TYPE.to_string(),
                    state,
                };
                let _ = self.notify_upward(device_state_bo)?;
                debug!(LOG_TAG, "the remote controller pressed, number: {}", pressed_num);
            }
        }
        Ok(())
    }
}

impl UpwardSendable for SerialRemoteController {
    fn get_upward_channel(&self) -> &Sender<DeviceStateDto> {
        return &self.upward_channel;
    }
}