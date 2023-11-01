//! 设备管理器
//! - 管理设备列表
//! - 管理设备与外部模块的通信
use std::collections::HashMap;
use std::error::Error;
use serde_json::{Value, Map};

use crate::common::dao::Dao;
use super::device_dao::DeviceDao;
use crate::common::http;
use crate::entity::po::device_po::DevicePo;
use crate::entity::bo::state_bo::StateBo;
use crate::{info, warn, error, trace, debug};


// 设备更新地址
const UPDATE_CONFIG_URL: &str = "api/v1.2/device";
const LOG_TAG : &str = "DeviceManager";


pub struct DeviceManager {
    device_dao: DeviceDao,
    cache_map: HashMap<String, DevicePo>,
    listener_map: HashMap<String, Vec<Box<dyn Fn(&Box<dyn StateBo>)>>>
}

impl DeviceManager {
    pub fn new() -> Self {
        DeviceManager{
            device_dao: DeviceDao::new(),
            cache_map: HashMap::new(),
            listener_map: HashMap::new(),
        }
    }

    /// 注册设备通知器
    pub fn register_listener(&mut self, device_id: &str, func: Box<dyn Fn(&Box<dyn StateBo>)>) {
        let listeners = self.listener_map.entry(device_id.to_string()).or_insert(Vec::new());
        listeners.push(func);
    }

    /// 发布设备状态通知
    pub fn notify(&self, device_id: &str, state_bo: &Box<dyn StateBo>) {
        if let Some(listeners) = self.listener_map.get(device_id) {
            for listener in listeners {
                listener(state_bo);
            }
        }
    }

    /// 系统初始化
    pub async fn startup(&mut self) -> Result<(), Box<dyn Error>> {
        self.device_dao.ensure_table_exist().await?;

        match self.get_remote().await {
            Ok(json_data) => {
                // 清空已有数据，并保存当前数据
                self.device_dao.clear_table().await?;
                self.write_to_db(json_data).await?;
                info!(LOG_TAG, "远程设备配置加载成功！");
            }
            Err(e) => {
                warn!(LOG_TAG, "无法获取远程设备配置文件，将使用本地缓存配置文件，错误信息：{}", e);
            }
        }

        self.load_from_db().await?;
        info!(LOG_TAG, "设备管理器已启动");
        Ok(())
    }

    /// 从远程获取设备配置文件
    async fn get_remote(&mut self) -> Result<Value, Box<dyn Error>>{
        http::api_get(UPDATE_CONFIG_URL).await
    }

    /// 将远程设备文件写入数据库
    async fn write_to_db(&self, json_data: Value) -> Result<(), Box<dyn Error>>{
        let device_list = json_data.get("list").unwrap().as_array().expect("list 未找到");
        for device in device_list {
            if let Some(device_po) = transform_json_data_to_po(device.clone()) {
                self.device_dao.add_device_config(device_po).await?;
            } else {
                warn!(LOG_TAG, "无法解析设备配置文件：{:?}", device);
            }
        }
        Ok(())
    }

    /// 从数据库中读取设备配置文件
    async fn load_from_db(&mut self) -> Result<(), Box<dyn Error>> {
        let device_config_po_list: Vec<DevicePo> = self.device_dao.get_all().await?;
        for device_config_po in device_config_po_list {
            self.cache_map.insert(device_config_po.device_id.clone(), device_config_po);
        }
        Ok(())
    }
    
}

fn transform_json_data_to_po(json_object: Value) -> Option<DevicePo> {
    let device_data = json_object.as_object()?;
    let config = device_data.get("config")?.as_object()?;
    let device_po = DevicePo {
        device_id: config.get("id")?.as_str()?.to_string(),
        device_class: config.get("class")?.as_str()?.to_string(),
        device_type: config.get("type")?.as_str()?.to_string(),
        name: config.get("name")?.as_str()?.to_string(),
        description: config.get("description")?.as_str()?.to_string(),
        room: config.get("room")?.as_str()?.to_string(),
        config: transform_device_config_obj_str(config)
    };
    Some(device_po)
}

// 辅助函数：构造一个配置文件 str 用于保存到数据库的 config 字段中
fn transform_device_config_obj_str(device_data: &Map<String, Value>) -> String {
    // 去除已经记录的字段
    let mut config = device_data.clone();
    config.remove("class");
    config.remove("type");
    config.remove("name");
    config.remove("description");
    config.remove("room");

    // 剩余字段导出为字符串
    let config_str = serde_json::to_string(&config).unwrap();
    config_str
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::logger::{init_logger};

    // 测试获取服务配置
    #[test] 
    fn test_get_device_config_from_remote() {
        init_logger();
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let mut manager = DeviceManager::new();
            manager.startup().await.unwrap();
        });
        info!(LOG_TAG, "测试完成");
    }
}