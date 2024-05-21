use std::sync::mpsc::Sender;

use crate::{
    common::error::{DeviceServerError, ServerErrorCode},
    entity::dto::{
        device_command_dto::{AudioParamsDto, CommandParamsEnum, DeviceCommandDto},
        mqtt_dto::{MqttDataDeviceCommandDto, MqttPayloadDto, MqttTopicDto},
    },
};

/// send device command to device manager
pub fn control_device_command(
    topic: MqttTopicDto,
    payload: MqttPayloadDto,
    command_tx: Sender<DeviceCommandDto>,
) -> Result<(), DeviceServerError> {
    let device_command_dto = make_device_command_dto(topic, payload)?;
    command_tx
        .send(device_command_dto)
        .map_err(|e| DeviceServerError {
            code: ServerErrorCode::MqttError,
            msg: format!("send device command dto error: {e}"),
        })?;
    Ok(())
}

/// make device command dto from topic and payload
fn make_device_command_dto(
    topic: MqttTopicDto,
    payload: MqttPayloadDto,
) -> Result<DeviceCommandDto, DeviceServerError> {
    let mut params = CommandParamsEnum::Empty;

    // get action from payload
    let action = payload.data["action"]
        .as_str()
        .ok_or(DeviceServerError {
            code: ServerErrorCode::MqttError,
            msg: format!("cannot get action from payload"),
        })?
        .to_string();

    let param = payload.data["param"].clone();

    // set pararms according to different device type
    if topic.device_type == Some("audio".to_string()) {
        let audio_params: AudioParamsDto =
            serde_json::from_value(param).map_err(|e| DeviceServerError {
                code: ServerErrorCode::MqttError,
                msg: format!("parse audio params from json to dto error: {e}"),
            })?;
        params = CommandParamsEnum::Audio(audio_params);
    }

    Ok(DeviceCommandDto {
        server_id: topic.server_id.unwrap(),
        device_id: topic.device_id.unwrap(),
        action: action,
        params: params,
    })
}
