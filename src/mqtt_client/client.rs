use tokio::sync::mpsc;

use crate::common::mqtt;
use crate::common::setting::Settings;
use crate::entity::mqtt::MqttMessageBo;
use super::protocol::Protocol;
use std::error::Error;
use std::result::Result;
use std::sync::Arc;

pub struct MqttClient {
    // mqtt 连接
    con: Option<mqtt::MqttConnection>,

    // 消息接收通道
    pub rx: Option<mpsc::Receiver<MqttMessageBo>>,

    // 消息转换对象
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

    /// 开启消息接受循环，处理接收到的消息
    pub async fn start(&mut self) -> Result<(), Box<dyn Error>> {
        let setting = Settings::get();
        let (tx, mut rx) = mpsc::channel(1);

        // 在新的线程里面创建 MqttConnection，使用 mpsc 通道做线程间通信
        let mut con = mqtt::MqttConnection::new(
            setting.mqtt.broker_host.as_str(), 
            setting.mqtt.broker_port.try_into().expect("mqtt broker 端口号定义错误"),
            setting.mqtt.client_id.as_str()
        );

        con.connect(tx).await.expect("mqtt 创建连接失败");
        self.con = Some(con);
        self.rx = Some(rx);

        log::info!("mqtt 连接成功 host: {} port: {}", setting.mqtt.broker_host, setting.mqtt.broker_port);

        self.subscribe_topics().await.expect("注册话题失败");
        Ok(())
    }

    /// 发布内容
    pub async fn publish(&self, topic: &str, payload: &str) -> Result<(), Box<dyn Error>> {
        match &self.con {
            Some(con) => {
                con.publish(topic, payload).await?;
            },
            None => {
                return Err("mqtt 连接未初始化".into());
            }
        }
        Ok(())
    }

    /// TODO: 发送心跳包
    pub async fn publish_heartbeat(&self) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    /// TODO: 发送设备状态变化通知
    pub async fn publish_status(&self) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    /// 发布离线消息
    pub async fn publish_offline(&self) -> Result<(), Box<dyn Error>> {
        match &self.con {
            Some(con) => {
                let topic = self.protocol.topic_self_declare("offline", None, None, None);
                let payload = self.protocol.payload_from_server(None, None, None, None);
                let json_str = payload.to_json()?;
                con.publish(topic.as_str(), json_str.as_str()).await?;
                Ok(())
            }
            None => Err("mqtt 连接未初始化".into())
        }
    }

    /// 注册预定义的 mqtt 话题
    pub async fn subscribe_topics(&mut self) -> Result<(), Box<dyn Error>> {
        match &self.con {
            Some(con) => {
                let setting = Settings::get();

                // 注册服务器话题
                let server_topic = format!(
                    "cmd/{}/{}/deviceserver/{}", 
                    setting.meta.application_name,
                    setting.meta.scenario_name,
                    setting.meta.server_name
                );
                con.subscribe(server_topic.as_str()).await?;

                // 注册设备话题
                let device_topic = format!(
                    "cmd/{}/{}/device/{}/+/+/+", 
                    setting.meta.application_name,
                    setting.meta.scenario_name,
                    setting.meta.server_name
                );

                con.subscribe(device_topic.as_str()).await?;

                // 注册广播话题
                let broadcast_topic = format!(
                    "broadcast/{}",
                    setting.meta.application_name
                );

                con.subscribe(broadcast_topic.as_str()).await?;

                Ok(())
            },
            None => {
                Err("mqtt 连接未初始化".into())
            }
        }
        
    }

}


// 单元测试部分

#[cfg(test)]
mod test {
    use super::*;
    use crate::common::logger::{init_logger};

    /// 测试接收一条消息
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
}