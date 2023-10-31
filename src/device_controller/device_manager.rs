//! 设备管理器
use std::collections::HashMap;
use std::error::Error;
use serde_json::{Value, Map};

use crate::common::dao::Dao;
use super::device_dao::DeviceDao;
use crate::common::http;
use crate::entity::po::DevicePo::DevicePo;
use crate::{info, warn, error, trace, debug};

// 设备更新地址
const UPDATE_CONFIG_URL: &str = "/api/v1.2/device";
const LOG_TAG : &str = "DeviceManager";


struct DeivceManager {
    device_dao: DeviceDao,
    cache: HashMap<String, DevicePo>
}

impl DeivceManager {
    fn new() -> Self {
        DeivceManager{
            device_dao: DeviceDao::new(),
            cache: HashMap::new()
        }
    }

    async fn init(&mut self) -> Result<(), Box<dyn Error>> {
        self.device_dao.ensure_table_exist().await?;
        Ok(())
    }

    /// 系统初始化
    async fn start(&mut self) -> Result<(), Box<dyn Error>> {
        match self.get_remote().await {
            Ok(json_data) => {
                // 清空已有数据，并保存当前数据
                self.device_dao.clear_table().await?;
                self.write_to_db(json_data).await?;
                info!(LOG_TAG, "远程设备配置加载成功！");
            }
            Err(e) => {
                warn!(LOG_TAG, "无法获取远程设备配置文件，错误信息：{}", e);
            }
        }
        self.load_from_db().await?;
        info!(LOG_TAG, "设备配置加载成功！");
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
            let device_data = device.as_object().expect("device 数据格式错误");
            let config = device_data.get("config").unwrap().as_object().expect("config 数据格式错误");
            let device_po = DevicePo {
                device_id: config.get("id").unwrap().as_str().unwrap().to_string(),
                device_class: config.get("class").unwrap().as_str().unwrap().to_string(),
                device_type: config.get("type").unwrap().as_str().unwrap().to_string(),
                name: config.get("name").unwrap().as_str().unwrap().to_string(),
                description: config.get("description").unwrap().as_str().unwrap().to_string(),
                room: config.get("room").unwrap().as_str().unwrap().to_string(),
                config: transform_device_config_obj_str(config)
            };
            self.device_dao.add_device_config(device_po).await?;
        }
        Ok(())
    }

    /// 从数据库中读取设备配置文件
    async fn load_from_db(&mut self) -> Result<(), Box<dyn Error>> {
        let device_config_po_list: Vec<DevicePo> = self.device_dao.get_all().await?;
        for device_config_po in device_config_po_list {
            self.cache.insert(device_config_po.device_id.clone(), device_config_po);
        }
        Ok(())
    }
    
}

// 构造一个配置文件 str 用于保存到数据库的 config 字段中
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
            let mut manager = DeivceManager::new();
            manager.init().await.unwrap();
            manager.get_remote().await.unwrap();
        });
        info!(LOG_TAG, "测试完成");
    }
}