use std::sync::mpsc::Sender;

use crate::driver::device::audio_output::{AudioOutput, ChannelEnum};
use crate::entity::dto::device_meta_info_dto::DeviceMetaInfoDto;
use crate::entity::dto::device_state_dto::DeviceStateDto;
use crate::{common::error::DriverError};
use crate::util::json;

pub fn make( device_info: &DeviceMetaInfoDto, report_tx: Sender<DeviceStateDto>) -> Result<AudioOutput, DriverError> {
    let soundcard_id = json::get_config_str(&device_info.config, "soundcard_id")?;
    let channel = json::get_config_str(&device_info.config, "channel")?;
    let channel_enum: ChannelEnum;
    if channel == "left" {
        channel_enum = ChannelEnum::Left;
    } else if channel == "right" {
        channel_enum = ChannelEnum::Right;
    } else {
        return Err(DriverError(format!("device factory: invalid channel str, channel: {}, device_id: {}", channel, device_info.device_id)));
    }
    let obj = AudioOutput::new(
        device_info.device_id.as_str(), 
        soundcard_id.as_str(),
        channel_enum,
        report_tx
    ); 
    Ok(obj)
}