use serde_json::{Result, Value};
use crate::util::time::get_timestamp;


/// mqtt Payload 对象，应用于和 mqtt 服务器的传输中
pub struct MqttPayloadDto {
    pub code: i32,
    pub msg: String,
    pub source_type: String,
    pub source_id: String,
    pub target_type: String,
    pub target_id: String,
    pub session_id: String,
    pub timestamp: f64,  // epoch 时间戳
    pub data: Value,
}

impl Default for MqttPayloadDto {
    fn default() -> Self {
        MqttPayloadDto {
            code: 200,
            msg: "ok".to_string(),
            source_type: "".to_string(),
            source_id: "".to_string(),
            target_type: "".to_string(),
            target_id: "".to_string(),
            session_id: "".to_string(),
            timestamp: get_timestamp(),
            data: Value::Null,
        }
    }
}


/// mqtt Topic 字符串中的信息
pub struct MqttTopicInfoBo {
    pub command: String,
    pub application: String,
    pub scenario: Option<String>,
    pub server_type: Option<String>,
    pub server_id: Option<String>,
    pub room_name: Option<String>,
    pub device_type: Option<String>,
    pub device_id: Option<String>,
}