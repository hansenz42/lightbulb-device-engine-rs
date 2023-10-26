//! 设备管理器
use std::error::Error;
use serde_json::{Value, Map};

use crate::common::dao::Dao;
use super::device_dao::DeviceDao;
use crate::common::http;
use crate::entity::po::DevicePo::DevicePo;


// 设备更新地址
const UPDATE_CONFIG_URL: &str = "/api/v1.2/device";


struct DeivceManager {
    device_dao: DeviceDao
}

impl DeivceManager {
    fn new() -> Self {
        DeivceManager{
            device_dao: DeviceDao::new()
        }
    }

    /// 系统初始化
    async fn init(&self) -> Result<(), Box<dyn Error>> {
        self.device_dao.ensure_table_exist().await?;
        Ok(())
    }

    /// 从远程获取设备配置文件
    async fn get_device_config_from_remote(&self) -> Result<(), Box<dyn Error>>{
        let result = http::api_get(UPDATE_CONFIG_URL).await?;
        // 先清除表中的数据再写新数据
        self.device_dao.clear_table().await?;
        self.write_config_to_local_cache(result).await?;
        Ok(())
    }

    /// 将远程设备文件写入数据库
    async fn write_config_to_local_cache(&self, json_data: Value) -> Result<(), Box<dyn Error>>{
        let device_list = json_data.get("list").unwrap().as_array().expect("list 未找到");
        for device in device_list {
            let device_data = device.as_object().expect("device 数据格式错误");
            let config = device_data.get("config").unwrap().as_object().expect("config 数据格式错误");
            let device_po = DevicePo {
                device_class: config.get("class").unwrap().as_str().unwrap().to_string(),
                device_type: config.get("type").unwrap().as_str().unwrap().to_string(),
                name: config.get("name").unwrap().as_str().unwrap().to_string(),
                description: config.get("description").unwrap().as_str().unwrap().to_string(),
                room: config.get("room").unwrap().as_str().unwrap().to_string(),
                config: construct_device_config_obj_str(config)
            };
            self.device_dao.add_device_config(device_po).await?;
        }
        Ok(())
    }
    
}

// 构造一个配置文件 str 用于保存到数据库的 config 字段中
fn construct_device_config_obj_str(device_data: &Map<String, Value>) -> String {
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
            let manager = DeivceManager::new();
            manager.init().await.unwrap();
            manager.get_device_config_from_remote().await.unwrap();
        });
        log::info!("测试完成");
    }
}