//! MQTT 服务连接器

use paho_mqtt;

struct MqttConnection {
    /// 远程服务器地址
    host: String,

    /// 端口
    port: u16,

    /// 连接客户端对象
    client: Option<paho_mqtt::AsyncClient>,
}

impl MqttConnection {
    pub fn new(host: &str, port: u16) -> Self {
        MqttConnection {
            host: host.to_string(),
            port,
            client: None
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
            log::error!("MQTT 未连接");
        }

        Ok(())
    }

    // 订阅、侦听消息
    
}