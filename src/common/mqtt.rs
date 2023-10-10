//! MQTT 服务连接器

use paho_mqtt;
use futures::{executor::block_on, stream::StreamExt};
use crate::entity::mqtt;

pub struct MqttConnection {
    /// 远程服务器地址
    host: String,

    /// 端口
    port: u16,

    /// 连接客户端对象
    client: Option<paho_mqtt::AsyncClient>,

    // 消息发送通道
    tx: tokio::sync::mpsc::Sender<mqtt::MqttMessageBo>,
}

impl MqttConnection {
    pub fn new(host: &str, port: u16, tx: tokio::sync::mpsc::Sender<mqtt::MqttMessageBo>) -> Self {
        MqttConnection {
            host: host.to_string(),
            port,
            client: None,
            tx,
        }
    }

    /// 连接到 MQTT 服务器
    pub async fn connect(&mut self) -> Result<(), paho_mqtt::Error> {
        let client = paho_mqtt::AsyncClient::new(format!("tcp://{}:{}", self.host.as_str(), self.port))?;
        
        let conn_opts = paho_mqtt::ConnectOptionsBuilder::new() 
            .keep_alive_interval(std::time::Duration::from_secs(20))
            .clean_session(true)
            .finalize();

        if let Err(e) = client.connect(conn_opts).await {
            log::error!("连接到 MQTT 服务器失败: {:?}", e);
            return Err(e);
        }

        self.client = Some(client);
        
        Ok(())
    }

    /// 连接到 MQTT 服务器（异步封装）
    

    /// 发布消息
    pub async fn publish(&self, topic: &str, payload: &str) -> Result<(), paho_mqtt::Error> {
        let msg = paho_mqtt::Message::new(topic, payload, 0);
        if let Some(client) = &self.client {
            client.publish(msg).await?;
        } else {
            log::error!("mqtt 消息发布失败，未连接");
        }

        Ok(())
    }

    /// 订阅消息
    pub async fn subscribe(&self, topic: &str) -> Result<(), paho_mqtt::Error> {
        if let Some(client) = &self.client {
            client.subscribe(topic, 0).await?;
        } else {
            log::error!("mqtt 消息订阅失败，未连接");
        }

        Ok(())
    }
    
    /// 开启消息循环
    pub async fn start(&mut self) -> Result<(), paho_mqtt::Error> {
        if let Some(client) = &mut self.client {
            let mut strm: paho_mqtt::AsyncReceiver<Option<paho_mqtt::Message>> = client.get_stream(25);
            while let Some(msg) = strm.next().await {
                if let Some(msg) = msg {
                    println!("payload: {:?}", msg.payload_str());
                    println!("topic: {:?}", msg.topic());
                    let message_bo = mqtt::MqttMessageBo {
                        topic: msg.topic().to_string(),
                        payload: msg.payload_str().to_string(),
                    };
                    // 向通道中发送数据
                    self.tx.send(message_bo).await.expect("mqtt 发送消息失败，通道错误");
                }
            }
        } else {
            log::error!("MQTT 未连接");
        }

        Ok(())
    }
}


#[cfg(test)]
mod test {
    use tokio::sync::mpsc;

    use super::*;
    use crate::common::logger::{init_logger};

    #[test]
    fn test_mqtt_connection() {
        init_logger();
        let rt = tokio::runtime::Runtime::new().unwrap();
        let (tx, mut rx) = mpsc::channel(1);
        let mut conn = MqttConnection::new("127.0.0.1", 1883, tx);
        rt.spawn(async move {
            conn.connect().await.unwrap();
            conn.subscribe("test").await.unwrap();
            conn.start().await;
        });
        let message_bo = rx.blocking_recv();
        println!("received: {:?}", message_bo);
        
    }
}