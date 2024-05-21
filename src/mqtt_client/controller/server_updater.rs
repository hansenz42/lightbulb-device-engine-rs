//! update the file manager

use crate::{
    common::error::DeviceServerError,
    entity::dto::mqtt_dto::{MqttPayloadDto, MqttTopicDto}, file_controller::file_controller::FileController,
};

/// update file controller data
pub fn update(topic: MqttTopicDto, payload: MqttPayloadDto) -> Result<(), DeviceServerError> {
    let file_controller = FileController::get();
    file_controller.update()?;
    Ok(())
}
