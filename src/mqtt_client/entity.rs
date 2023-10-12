use serde_json::{Result, Value};
use crate::util::time::get_timestamp;
use crate::util::gen_id::generate_uuid;
use crate::common::setting::Settings;


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

impl MqttPayloadDto {
    pub fn new(
        code: Option<i32>,
        msg: Option<String>, 
        source_type: Option<String>,
        source_id: Option<String>,
        target_type: Option<String>,
        target_id: Option<String>,
        session_id: Option<String>,
        timestamp: Option<f64>,
        data: Option<Value>,
    ) -> Self {
        let settings = Settings::get();
        MqttPayloadDto {
            code: match code {
                Some(code) => code,
                None => 200,
            },
            msg: match msg {
                Some(msg) => msg,
                None => String::from(""),
            },
            source_type: match source_type {
                Some(source_type) => source_type,
                None => settings.server.server_type.clone(),
            },
            source_id: match source_id {
                Some(source_id) => source_id,
                None => settings.server.server_id.clone(),
            },
            target_type: match target_type {
                Some(target_type) => target_type,
                None => String::from(""),
            },
            target_id: match target_id {
                Some(target_id) => target_id,
                None => String::from(""),
            },
            session_id: match session_id {
                Some(session_id) => session_id,
                None => generate_uuid(),
            },
            timestamp: match timestamp {
                Some(timestamp) => timestamp,
                None => get_timestamp(),
            },
            data: match data {
                Some(data) => data,
                None => Value::Null,
            },
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