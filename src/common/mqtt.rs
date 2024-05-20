//! MQTT 服务连接器

use std::{pin::Pin, sync::{mpsc, Arc}};

use paho_mqtt;
use crate::{debug, error, info, trace, warn};

const LOG_TAG : &str = "mqtt";

pub struct MqttConnection {
    /// 远程服务器地址
    host: String,

    /// 端口
    port: u16,

    // client_id
    client_id: String,

    /// 连接客户端对象
    client: Option<paho_mqtt::AsyncClient>
}

impl MqttConnection {
    pub fn new(host: &str, port: u16, client_id: &str) -> Self {
        MqttConnection {
            host: host.to_string(),
            port,
            client_id: client_id.to_string(),
            client: None
        }
    }

    pub fn set_callback(&mut self, callback: impl FnMut(&paho_mqtt::AsyncClient, Option<paho_mqtt::Message>) + Send + 'static) {
        self.client.as_mut().unwrap().set_message_callback(callback);
    }

    /// init mqtt server
    /// 传入的 tx 为发送消息的 mpsc 通道，目前支持标准库的多发单收
    pub fn connect(&mut self) -> Result<(), paho_mqtt::Error> {
        let create_opts = paho_mqtt::CreateOptionsBuilder::new()
            .server_uri(format!("tcp://{}:{}", self.host.as_str(), self.port))
            .client_id(self.client_id.as_str())
            .finalize();
        
        let client = paho_mqtt::AsyncClient::new(create_opts)?;
        
        let conn_opts = paho_mqtt::ConnectOptionsBuilder::new() 
            .keep_alive_interval(std::time::Duration::from_secs(20))
            .clean_session(true)
            .finalize();

        client.set_connection_lost_callback(|_cli| {
            error!(LOG_TAG, "*** mqtt Connection lost ***");
        });

        // client.set_message_callback(move |_cli, msg| {
        //     if let Some(msg) = msg {
        //         debug!(LOG_TAG, "received message! content {}", msg.payload_str());
        //         tx.send(MqttMessageDto {
        //             topic: msg.topic().to_string(),
        //             payload: msg.payload_str().to_string()
        //         }).expect("mqtt message sending failed");
        //     }
        // });

        if let Err(e) = client.connect(conn_opts).wait() {
            error!(LOG_TAG, "cannot connect to mqtt server: {:?}", e);
            return Err(e);
        }

        self.client = Some(client);
        
        Ok(())
    }
    

    pub fn publish(&self, topic: &str, payload: &str) -> Result<(), paho_mqtt::Error> {
        let msg = paho_mqtt::Message::new(topic, payload, 0);
        if let Some(client) = &self.client {
            client.publish(msg).wait()?;
        } else {
            error!(LOG_TAG, "mqtt publish failed, no connection");
        }

        Ok(())
    }

    pub fn subscribe(&self, topic: &str) -> Result<(), paho_mqtt::Error> {
        if let Some(client) = &self.client {
            client.subscribe(topic, 0).wait()?;
        } else {
            error!(LOG_TAG, "mqtt subscribe failed, no connection");
        }

        Ok(())
    }
}


#[cfg(test)]
mod test {
    use super::*;
    use crate::common::logger::init_logger;
    use std::env;

    // 单次接收数据测试
    #[test]
    fn test() {
        init_logger().expect("初始化日志失败");
        let mut mqtt = MqttConnection::new("127.0.0.1", 1883, "test_client");
        mqtt.connect().unwrap();
        mqtt.subscribe("test").unwrap();
        println!("wait for message");
        println!("进程已结束，进入等待……");
    }
}