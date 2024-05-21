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

use super::{controller::device_commander::device_command_controller, protocol::Protocol};

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
        device_command_controller(topic_dto, payload_dto, command_tx)?;
    } else {
        // 3.2 if there is not device_id, which means the target of the message is server
    }

    Ok(())
}