//! get message from mqtt client, then dispatch message to different controller

use std::sync::mpsc::Sender;

use paho_mqtt::{AsyncClient, Message};

use crate::{
    common::error::{DeviceServerError, ServerErrorCode},
    entity::dto::{
        device_command_dto::{AudioParamsDto, CommandParamsEnum, DeviceCommandDto},
        mqtt_dto::{MqttDataDeviceCommandDto, MqttPayloadDto, MqttTopicDto},
    },
};

use super::protocol::Protocol;

pub fn on_message(
    msg: Message,
    command_tx: Sender<DeviceCommandDto>,
) -> Result<(), DeviceServerError> {
    // 1. parse topic
    let topic_dto = Protocol::parse_topic(msg.topic()).map_err(|e| DeviceServerError {
        code: ServerErrorCode::MqttError,
        msg: format!("parse mqtt topic error: {e}"),
    })?;

    // 2. parse payload
    let payload_dto =
        MqttPayloadDto::from_json(msg.payload_str().to_string().as_str()).map_err(|e| {
            DeviceServerError {
                code: ServerErrorCode::MqttError,
                msg: format!("parse mqtt payload error: {e}"),
            }
        })?;

    if let Some(ref device_id) = topic_dto.device_id {
        // 3.1 if there is device_id in topic_dto, which means that is a device command
        let device_command_dto = make_device_command_dto(topic_dto, payload_dto)?;
        command_tx.send(device_command_dto)
            .map_err(|e| DeviceServerError {code: ServerErrorCode::MqttError, msg: format!("send device command dto error: {e}")})?;
    } else {
        // 3.2 if there is not device_id, which means the target of the message is server
    }

    Ok(())
}

fn make_device_command_dto(
    topic: MqttTopicDto,
    payload: MqttPayloadDto,
) -> Result<DeviceCommandDto, DeviceServerError> {

    let mut params = CommandParamsEnum::Empty;

    // get action from payload
    let action = payload.data["action"].as_str()
        .ok_or(DeviceServerError {
            code: ServerErrorCode::MqttError,
            msg: format!("cannot get action from payload"),
        })?.to_string();

    // set pararms according to different device type
    if topic.device_type == Some("audio".to_string()) {
        let audio_params: AudioParamsDto = serde_json::from_value(payload.data).map_err(|e| DeviceServerError {
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