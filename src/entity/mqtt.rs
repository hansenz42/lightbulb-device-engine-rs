//! 与 mqtt 相关的结构体

#[derive(Debug, Clone)]
pub struct MqttMessageBo {
    pub topic: String,
    pub payload: String,
}