use tokio::sync::mpsc;

use crate::common::mqtt;
use crate::common::setting::Settings;
use crate::entity::mqtt::MqttMessageBo;
use super::protocol::Protocol;
use std::error::Error;
use std::result::Result;

struct MqttClient {
    // mqtt 连接实体类
    con: mqtt::MqttConnection,

    // 消息接收通道
    rx: mpsc::Receiver<MqttMessageBo>,

    // 消息转换对象
    protocol: Protocol
}

impl MqttClient {
    fn new() -> Self {
        let setting = Settings::get();
        let (tx, mut rx) = mpsc::channel(1);
        MqttClient {
            con: mqtt::MqttConnection::new(
                setting.mqtt.broker_host.as_str(), 
                setting.mqtt.broker_port.try_into().expect("mqtt broker port error"),
                tx
            ),
            rx,
            protocol: Protocol::new()
        }
    }

    /// 发布内容
    pub async fn publish(&self, topic: &str, payload: &str) -> Result<(), Box<dyn Error>> {
        self.con.publish(topic, payload).await?;
        Ok(())
    }

    /// 注册预定义的 mqtt 话题
    pub async fn subscribe_topics(&mut self) -> Result<(), Box<dyn Error>> {
        let setting = Settings::get();

        // 注册服务器话题
        let server_topic = format!(
            "cmd/{}/{}/deviceserver/{}", 
            setting.meta.application_name,
            setting.meta.scenario_name,
            setting.meta.server_name
        );
        self.con.subscribe(server_topic.as_str()).await?;

        // 注册设备话题
        let device_topic = format!(
            "cmd/{}/{}/device/{}/+/+/+", 
            setting.meta.application_name,
            setting.meta.scenario_name,
            setting.meta.server_name
        );

        self.con.subscribe(device_topic.as_str()).await?;

        // 注册广播话题
        let broadcast_topic = format!(
            "broadcast/{}",
            setting.meta.application_name
        );
        self.con.subscribe(broadcast_topic.as_str()).await?;

        Ok(())
    }
}