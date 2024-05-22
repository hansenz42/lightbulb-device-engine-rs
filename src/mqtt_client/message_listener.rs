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

use super::{controller::device_commander::control_device_command, protocol::Protocol, controller::server_updater::update};

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
        control_device_command(topic_dto, payload_dto, command_tx)?;
    } else {
        // 3.2 if there is not device_id, which means the target of the message is server
        // updating file controller data
        let action = payload_dto.data["action"].as_str().ok_or(
            DeviceServerError {
                code: ServerErrorCode::MqttError,
                msg: "action not found".to_string(),
            },
        )?;
        if action == "update" {
            update(topic_dto, payload_dto)?;
        }
    }

    Ok(())
}
