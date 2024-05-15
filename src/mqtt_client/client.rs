use tokio::sync::mpsc;

use crate::common::error::{DeviceServerError, ServerErrorCode};
use crate::common::mqtt;
use crate::common::setting::Settings;
use crate::entity::dto::server_state_dto::ServerStateDto;
use crate::entity::mqtt::MqttMessageBo;
use super::protocol::Protocol;
use std::error::Error;
use std::result::Result;
use std::sync::Arc;
use crate::entity::dto::device_state_dto::DeviceStateDto;

pub struct MqttClient {
    // mqtt connection
    con: Option<mqtt::MqttConnection>,

    // message recv channel
    pub rx: Option<mpsc::Receiver<MqttMessageBo>>,

    // message protocol
    protocol: Protocol
}

impl MqttClient {
    pub fn new() -> Self {
        let setting = Settings::get();
        MqttClient {
            con: None,
            rx: None,
            protocol: Protocol::new()
        }
    }

    /// start event loop thread and communicate with the flow server
    pub async fn start(&mut self) -> Result<(), DeviceServerError> {
        let setting = Settings::get();
        let (tx, mut rx) = mpsc::channel(1);

        // create new MqttConnection in the thread，use mpsc channel to communicate between threads
        let mut con = mqtt::MqttConnection::new(
            setting.mqtt.broker_host.as_str(), 
            setting.mqtt.broker_port.try_into().expect("mqtt broker port data type error, is not u16"),
            setting.mqtt.client_id.as_str()
        );

        con.connect(tx).await
            .map_err(|e| DeviceServerError {code: ServerErrorCode::MqttError, msg: format!("mqtt connect error: {e}")} )?;
        self.con = Some(con);
        self.rx = Some(rx);

        log::info!("mqtt connect successful host: {} port: {}", setting.mqtt.broker_host, setting.mqtt.broker_port);

        self.subscribe_topics().await.expect("subscribe topics failed");
        Ok(())
    }

    /// according topic and payload to publish message
    pub async fn publish(&self, topic: &str, payload: &str) -> Result<(), DeviceServerError> {
        match &self.con {
            Some(con) => {
                con.publish(topic, payload).await
                    .map_err(|e| DeviceServerError {code: ServerErrorCode::MqttError, msg: format!("mqtt publish error: {e}")} )?;
            },
            None => {
                return Err(DeviceServerError {code: ServerErrorCode::MqttError, msg: format!("mqtt publish error: not connect")});
            }
        }
        Ok(())
    }

    pub async fn publish_heartbeat(&self, server_state: ServerStateDto) -> Result<(), DeviceServerError> {
        Ok(())
    }

    /// publish device status message
    pub async fn publish_status(&self, state_dto: DeviceStateDto) -> Result<(), DeviceServerError> {
        let topic = self.protocol.topic_self_declare("status", None, Some(state_dto.device_class.clone()), Some(state_dto.device_id.clone()));

        let payload_content = serde_json::to_value(state_dto)
            .map_err(|e| DeviceServerError {code: ServerErrorCode::MqttError, msg: format!("cannot publish status message, transform state bo to json failed, json error: {e}")})?;
        let payload = self.protocol.payload_from_server( Some(payload_content), None, None, None);

        let json_str = payload.to_json()
            .map_err(|e| DeviceServerError {code: ServerErrorCode::MqttError, msg: format!("cannot publish status message, transform from payload to json failed, json error: {e}")})?;
        self.publish(topic.as_str(), json_str.as_str()).await?;
        Ok(())
    }

    /// publish offline message
    pub async fn publish_offline(&self) -> Result<(), DeviceServerError> {
        match &self.con {
            Some(con) => {
                let topic = self.protocol.topic_self_declare("offline", None, None, None);
                let payload = self.protocol.payload_from_server(None, None, None, None);
                let json_str = payload.to_json()
                    .map_err(|e| DeviceServerError {code: ServerErrorCode::MqttError, msg: format!("cannot publish offline message, transform from payload to json failed, json error: {e}")})?;
                self.publish(topic.as_str(), json_str.as_str()).await?;
                Ok(())
            }
            None => Err(DeviceServerError {code: ServerErrorCode::MqttError, msg: format!("mqtt publish error: not connect")})
        }
    }

    /// register predefined topics
    pub async fn subscribe_topics(&mut self) -> Result<(), DeviceServerError> {
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
                self.subscribe(server_topic.as_str()).await?;

                // cmd for device topic 
                let device_topic = format!(
                    "cmd/{}/{}/device/{}/+/+/+", 
                    setting.meta.application_name,
                    setting.meta.scenario_name,
                    setting.meta.server_name
                );
                self.subscribe(device_topic.as_str()).await?;

                // broadcast topic
                let broadcast_topic = format!(
                    "broadcast/{}",
                    setting.meta.application_name
                );
                self.subscribe(broadcast_topic.as_str()).await?;

                Ok(())
            },
            None => {
                Err(DeviceServerError {code: ServerErrorCode::MqttError, msg: format!("mqtt subscribe error: not connect")})
            }
        }
    }

    async fn subscribe(&self, topic: &str) -> Result<(), DeviceServerError> {
        match &self.con {
            Some(con) => {
                con.subscribe(topic).await
                    .map_err(|e| DeviceServerError {code: ServerErrorCode::MqttError, msg: format!("mqtt subscribe error: {e}")} )?;
                Ok(())
            },
            None => {
                Err(DeviceServerError {code: ServerErrorCode::MqttError, msg: format!("mqtt subscribe error: not connect")})
            }
        }
    }

}


// 单元测试部分
#[cfg(test)]
mod test {
    use rodio::Device;

    use super::*;
    use crate::{common::logger::init_logger, entity::dto::device_state_dto::{DoStateDto, StateDtoEnum}};

    /// recv message test
    #[test]
    fn test() {
        init_logger();
        let rt = tokio::runtime::Runtime::new().unwrap();
        let mut client = MqttClient::new();

        rt.block_on(async move {
            client.start().await.unwrap();
            client.publish("test", "from rust client").await.unwrap();

            match &mut client.rx {
                Some(rx) => {
                    println!("等待消息一条");
                    let message_bo = rx.recv().await;
                    println!("接收到消息: {:?}", message_bo);
                },
                None => {
                    println!("未初始化");
                }
            }
        });
    }

    #[test]
    fn test_publish_status() {
        init_logger();
        println!("publish status testing: will use do state dto");
        let rt = tokio::runtime::Runtime::new().unwrap();
        let mut client = MqttClient::new();

        rt.block_on(async move {
            client.start().await.unwrap();
            client.publish_status(
                DeviceStateDto {
                    device_id: "test".to_string(),
                    device_class: "test".to_string(),
                    device_type: "test".to_string(),
                    state: StateDtoEnum::Do(
                        DoStateDto {
                            on: true
                        }
                    ) 
                }
            ).await.unwrap();
        });
    }
}