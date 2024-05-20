use std::os::linux::raw;
use std::sync::mpsc::{self, Receiver, Sender};

use super::protocol::Protocol;
use crate::common::error::{DeviceServerError, ServerErrorCode};
use crate::common::mqtt;
use crate::common::setting::Settings;
use crate::entity::dto::device_command_dto::DeviceCommandDto;
use crate::entity::dto::device_state_dto::DeviceStateDto;
use crate::entity::dto::mqtt_dto::{MqttDataDeviceCommandDto, MqttPayloadDto};
use crate::entity::dto::server_state_dto::ServerStateDto;
use crate::{debug, error, info};
use std::result::Result;

const LOG_TAG: &str = "mqtt_client";

pub struct MqttClient {
    // mqtt connection
    con: Option<mqtt::MqttConnection>,

    // message protocol
    protocol: Protocol,
}

impl MqttClient {
    pub fn new() -> Self {
        let setting = Settings::get();
        MqttClient {
            con: None,
            protocol: Protocol::new(),
        }
    }

    /// start event loop thread and communicate with the flow server
    /// mqtt start will return message receiver channel
    /// command_tx: for sending device command dto
    pub fn start(&mut self, command_tx: Sender<DeviceCommandDto>) -> Result<(), DeviceServerError> {
        let setting = Settings::get();

        // create new MqttConnection in the thread，use mpsc channel to communicate between threads
        let mut con = mqtt::MqttConnection::new(
            setting.mqtt.broker_host.as_str(),
            setting
                .mqtt
                .broker_port
                .try_into()
                .expect("mqtt broker port data type error, is not u16"),
            setting.mqtt.client_id.as_str(),
        );

        con.connect().map_err(|e| DeviceServerError {
            code: ServerErrorCode::MqttError,
            msg: format!("mqtt connect error: {e}"),
        })?;

        con.set_callback(move |_cli, msg| {
            if let Some(msg) = msg {
                debug!(LOG_TAG, "received mqtt msg, content: {}", msg.payload_str());

                // 1. parse topic
                match Protocol::parse_topic(msg.topic()) {
                    Ok(topic_dto) => {
                        let raw_payload = msg.payload_str().to_string();

                        // 2. parse mqtt payload
                        match MqttPayloadDto::from_json(raw_payload.as_str()) {
                            Ok(payload_dto) => {
                                debug!(LOG_TAG, "parse mqtt payload: {:?}", payload_dto);

                                // device command message: if there is device id in topic
                                if let Some(device_id) = topic_dto.device_id {
                                    // 2.1 parse device command dto
                                    match serde_json::from_value::<MqttDataDeviceCommandDto>(payload_dto.data) {
                                        Ok(mqtt_device_command_dto) => {
                                                    
                                            let device_command_dto = DeviceCommandDto {
                                                server_id: topic_dto.server_id.unwrap(),
                                                device_id: device_id,
                                                action: mqtt_device_command_dto.action,
                                                params: mqtt_device_command_dto.params,
                                            };
                                            command_tx.send(device_command_dto).unwrap();

                                        }
                                        Err(e) => {
                                            error!(LOG_TAG, "parse mqtt payload from json to dto (MqttDataDeviceCommandDto) error: {:?}", e);
                                            return;
                                        }
                                    }
                                }

                            }

                            Err(e) => {
                                error!(LOG_TAG, "parse mqtt payload from json to dto (MqttPayloadDto) error: {:?}", e);
                                return;
                            }
                        }
                    }
                    Err(e) => {}
                }
            }
        });

        self.con = Some(con);
        log::info!(
            "mqtt connect successful host: {} port: {}",
            setting.mqtt.broker_host,
            setting.mqtt.broker_port
        );

        self.subscribe_topics().expect("subscribe topics failed");

        Ok(())
    }

    /// according topic and payload to publish message
    pub fn publish(&self, topic: &str, payload: &str) -> Result<(), DeviceServerError> {
        match &self.con {
            Some(con) => {
                con.publish(topic, payload).map_err(|e| DeviceServerError {
                    code: ServerErrorCode::MqttError,
                    msg: format!("mqtt publish error: {e}"),
                })?;
            }
            None => {
                return Err(DeviceServerError {
                    code: ServerErrorCode::MqttError,
                    msg: format!("mqtt publish error: not connect"),
                });
            }
        }
        Ok(())
    }

    /// publish heartbeat message
    pub fn publish_heartbeat(&self, server_state: ServerStateDto) -> Result<(), DeviceServerError> {
        // 1 make topic
        let topic = self.protocol.topic_self_declare("status", None, None);

        // 2 make payload
        let payload_content = serde_json::to_value(server_state)
            .map_err(|e| DeviceServerError {code: ServerErrorCode::MqttError, msg: format!("cannot publish heartbeat message, transform state bo to json failed, json error: {e}")})?;
        let payload = self
            .protocol
            .payload_from_server(Some(payload_content), None, None, None);

        // 3 payload to json string
        let json_str = payload.to_json()
            .map_err(|e| DeviceServerError {code: ServerErrorCode::MqttError, msg: format!("cannot publish heartbeat message, transform from payload to json failed, json error: {e}")})?;

        // 4 publish
        self.publish(topic.as_str(), json_str.as_str())?;

        Ok(())
    }

    /// publish device status message
    pub fn publish_status(&self, state_dto: DeviceStateDto) -> Result<(), DeviceServerError> {
        let topic = self.protocol.topic_self_declare(
            "status",
            Some(state_dto.device_class.clone()),
            Some(state_dto.device_id.clone()),
        );

        let payload_content = serde_json::to_value(state_dto).map_err(|e| DeviceServerError {
            code: ServerErrorCode::MqttError,
            msg: format!(
                "cannot publish status message, transform state bo to json failed, json error: {e}"
            ),
        })?;
        let payload = self
            .protocol
            .payload_from_server(Some(payload_content), None, None, None);

        let json_str = payload.to_json()
            .map_err(|e| DeviceServerError {code: ServerErrorCode::MqttError, msg: format!("cannot publish status message, transform from payload to json failed, json error: {e}")})?;
        self.publish(topic.as_str(), json_str.as_str())?;
        Ok(())
    }

    /// publish offline message
    pub fn publish_offline(&self) -> Result<(), DeviceServerError> {
        match &self.con {
            Some(con) => {
                let topic = self.protocol.topic_self_declare("offline", None, None);
                let payload = self.protocol.payload_from_server(None, None, None, None);
                let json_str = payload.to_json()
                    .map_err(|e| DeviceServerError {code: ServerErrorCode::MqttError, msg: format!("cannot publish offline message, transform from payload to json failed, json error: {e}")})?;
                self.publish(topic.as_str(), json_str.as_str())?;
                Ok(())
            }
            None => Err(DeviceServerError {
                code: ServerErrorCode::MqttError,
                msg: format!("mqtt publish error: not connect"),
            }),
        }
    }

    /// register predefined topics
    pub fn subscribe_topics(&mut self) -> Result<(), DeviceServerError> {
        match &self.con {
            Some(con) => {
                let setting = Settings::get();

                // cmd for device server topic
                let server_topic = format!(
                    "cmd/{}/{}/deviceserver/{}",
                    setting.meta.application_name,
                    setting.meta.scenario_name,
                    setting.meta.server_name
                );
                self.subscribe(server_topic.as_str())?;

                // cmd for device topic
                let device_topic = format!(
                    "cmd/{}/{}/device/{}/+/+/+",
                    setting.meta.application_name,
                    setting.meta.scenario_name,
                    setting.meta.server_name
                );
                self.subscribe(device_topic.as_str())?;

                // broadcast topic
                let broadcast_topic = format!("broadcast/{}", setting.meta.application_name);
                self.subscribe(broadcast_topic.as_str())?;

                Ok(())
            }
            None => Err(DeviceServerError {
                code: ServerErrorCode::MqttError,
                msg: format!("mqtt subscribe error: not connect"),
            }),
        }
    }

    fn subscribe(&self, topic: &str) -> Result<(), DeviceServerError> {
        match &self.con {
            Some(con) => {
                con.subscribe(topic).map_err(|e| DeviceServerError {
                    code: ServerErrorCode::MqttError,
                    msg: format!("mqtt subscribe error: {e}"),
                })?;
                Ok(())
            }
            None => Err(DeviceServerError {
                code: ServerErrorCode::MqttError,
                msg: format!("mqtt subscribe error: not connect"),
            }),
        }
    }
}

// 单元测试部分
#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        common::logger::init_logger,
        entity::dto::device_state_dto::{DoStateDto, StateDtoEnum},
    };

    /// recv message test
    #[test]
    fn test() {
        init_logger();
        let mut client = MqttClient::new();

        println!("等待消息一条");
        println!("接收到消息: {:?}", "test");
    }

    #[test]
    fn test_publish_status() {
        init_logger();
        println!("publish status testing: will use do state dto");
        let mut client = MqttClient::new();

        client
            .publish_status(DeviceStateDto {
                device_id: "test".to_string(),
                device_class: "test".to_string(),
                device_type: "test".to_string(),
                state: StateDtoEnum::Do(DoStateDto { on: true }),
            })
            .unwrap();
    }
}
