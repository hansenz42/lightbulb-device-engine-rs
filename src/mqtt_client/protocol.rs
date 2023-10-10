
use std::error::Error;

use super::entity::{MqttTopicInfoBo, MqttPayloadDto};
use crate::common::setting::Settings;

// 对 TopicInfo 结构体做二次包装，使其支持默认值的调用方法
struct TopicConfig(MqttTopicInfoBo);

impl Default for TopicConfig {
    // 使用 wrapper 包裹 TopicConfig 结构体，用于生成 Topic Config 的默认值
    fn default() -> Self {
        let setting = Settings::get();
        TopicConfig(MqttTopicInfoBo {
            command: "".to_string(),
            application: setting.meta.application_name.clone(),
            scenario: Some(setting.meta.scenario_name.clone()),
            server_type: Some(setting.server.server_type.clone()),
            server_id: Some(setting.server.server_id.clone()),
            room_name: None,
            device_type: None,
            device_id: None,
        })
    }
}

/// mqtt 协议处理工具类，将 bytestring 和已经定义好的结构体互相转换
pub struct Protocol {
    // 基本数据
    app_name: String,
    scenario_name: String,
    server_type: String,
    server_id: String
}

impl Protocol {
    pub fn new() -> Self {
        Protocol {
            app_name: Settings::get().meta.application_name.clone(),
            scenario_name: Settings::get().meta.scenario_name.clone(),
            server_type: Settings::get().server.server_type.clone(),
            server_id: Settings::get().server.server_id.clone()
        }
    }

    /// 将 topic 字符串解析为结构体
    fn parse_topic(topic_str: String) -> Result<MqttTopicInfoBo, Box<dyn Error>>{
        let topic_vec: Vec<&str> = topic_str.split("/").collect();
        let command = topic_vec[0].to_string();
        let application = topic_vec[1].to_string();
        let mut scenario = Option::None;
        let mut server_type = Option::None;
        let mut server_id = Option::None;
        let mut room_name = Option::None;
        let mut device_type = Option::None;
        let mut device_id = Option::None;

        if topic_vec.len() >= 3 {
            scenario = Some(topic_vec[2].to_string());
        }

        if topic_vec.len() >= 4 {
            server_type = Some(topic_vec[3].to_string());
            server_id = Some(topic_vec[4].to_string());
        }

        if topic_vec.len() >= 6 {
            room_name = Some(topic_vec[5].to_string());
            device_type = Some(topic_vec[6].to_string());
            device_id = Some(topic_vec[7].to_string());
        }

        Ok(MqttTopicInfoBo {
            command,
            application,
            scenario,
            server_type,
            server_id,
            room_name,
            device_type,
            device_id,
         })
    }

    /// 生成 topic String 字符串
    fn generate_topic (wrapper : TopicConfig) -> String {
        let mut topic = String::new();
        let bo = wrapper.0;
        topic.push_str(bo.command.as_str());
        topic.push_str("/");
        topic.push_str(bo.application.as_str());
        topic.push_str("/");
        if let Some(scenario) = bo.scenario {
            topic.push_str(scenario.as_str());
        }
        topic.push_str("/");
        if let Some(server_type) = bo.server_type {
            topic.push_str(server_type.as_str());
        }
        topic.push_str("/");
        if let Some(server_id) = bo.server_id {
            topic.push_str(server_id.as_str());
        }
        topic.push_str("/");
        if let Some(room_name) = bo.room_name {
            topic.push_str(room_name.as_str());
        }
        topic.push_str("/");
        if let Some(device_type) = bo.device_type {
            topic.push_str(device_type.as_str());
        }
        topic.push_str("/");
        if let Some(device_id) = bo.device_id {
            topic.push_str(device_id.as_str());
        }
        topic
    }

    /// 生成只有 command 的 topic 字符串
    pub fn topic_with_command(&self, command: &str) -> String {
        let mut config = TopicConfig::default();
        config.0.command = command.to_string();
        Self::generate_topic(config)
    }

    /// 发送给目标设备的 topic
    pub fn topic_with_target(&self, command: &str, server_type: &str, server_id: &str, room_name: Option<String>, device_type: Option<String>, device_id: Option<String>) -> String {
        let mut config = TopicConfig::default();
        config.0.command = command.to_string();
        config.0.server_type = Some(server_type.to_string());
        config.0.server_id = Some(server_id.to_string());
        config.0.room_name = room_name;
        config.0.device_type = device_type;
        config.0.device_id = device_id;
        Self::generate_topic(config)
    }

    /// 发送设备自我声明的 topic
    pub fn topic_self_declare(&self, command: &str, room_name: Option<String>, device_type: Option<String>, device_id: Option<String>) -> String {
        let mut config = TopicConfig::default();
        config.0.command = command.to_string();
        config.0.room_name = room_name;
        config.0.device_type = device_type;
        config.0.device_id = device_id;
        Self::generate_topic(config)
    }

    /// 发送错误消息数据
    pub fn error_message(&self, msg: String, session_id: String, target_type: String, target_id: String) -> MqttPayloadDto {
        let mut payload = MqttPayloadDto::default();
        payload.code = 500;
        payload.msg = msg;
        payload.session_id = session_id;
        payload.target_type = target_type;
        payload.target_id = target_id;
        payload
    }


    /// 发送参数错误消息
    pub fn param_fail_message(&self, msg: String, session_id: String, target_type: String, target_id: String) -> MqttPayloadDto {
        let mut payload = MqttPayloadDto::default();
        payload.code = 400;
        payload.msg = msg;
        payload.session_id = session_id;
        payload.target_type = target_type;
        payload.target_id = target_id;
        payload
    }

    /// 服务器发送的带数据消息
    pub fn message_from_server(&self, data: serde_json::Value, session_id: String, target_type: String, target_id: String) -> MqttPayloadDto {
        let mut payload = MqttPayloadDto::default();
        payload.data = data;
        payload.session_id = session_id;
        payload.target_type = target_type;
        payload.target_id = target_id;
        payload
    }

    
}