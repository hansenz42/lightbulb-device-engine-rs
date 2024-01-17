//! 设备管理器，主要实现两部分功能
//! - 设备配置信息管理：从远程服务器获得配置数据，并保存到本地缓存。优先下载远程数据，如果远程数据下载失败，则使用本地缓存
//! - 设备通信管理：双线程结构，一个负责上行通信，一个负责下行通信
//! - 下行通信线程管理设备实例，避免了实例在多线程传递的问题
use std::cell::RefCell;
use std::collections::HashMap;
use std::error::Error;
use serde_json::{Value, Map};

use crate::common::dao::Dao;
use crate::entity::bo::device_config_bo::{ConfigBo};
use crate::mqtt_client::client::MqttClient;
use super::device_dao::DeviceDao;
use crate::common::http;
use crate::entity::po::device_po::DevicePo;
use crate::entity::bo::device_state_bo::{DeviceStateBo, StateBoEnum};
use crate::entity::bo::device_command_bo::DeviceCommandBo;
use crate::{info, warn, error, trace, debug};
use std::thread;
use std::sync::{mpsc, Arc};
use crate::driver::traits::device::Device;
use crate::driver::device::device_enum::DeviceEnum;
use crate::driver::factory::device_factory::DeviceFactory;


// 设备列表更新地址
const UPDATE_CONFIG_URL: &str = "api/v1.2/device";
const LOG_TAG : &str = "DeviceManager";

/// 设备管理器
/// - 管理设备列表
/// - 管理设备与外部变量的通信
/// - 双线程架构，一个线程负责上行通信（到 mqtt），一个线程负责下行通信（到设备）
pub struct DeviceManager {
    device_dao: DeviceDao,
    pub config_map: HashMap<String, DevicePo>,
    
    // 上行：接收从 device 来的消息，发送到 mqtt
    upward_rx: mpsc::Receiver<DeviceStateBo>,
    
    // manager 名下的设备，clone 该 tx 即可向上发送数据
    upward_tx: mpsc::Sender<DeviceStateBo>,
    
    // 下行：从 mqtt 服务器接收到的消息，给设备的指令
    downward_rx: Option<mpsc::Receiver<DeviceCommandBo>>,
}

impl DeviceManager {
    pub fn new() -> Self {
        let (upward_tx, upward_rx) = mpsc::channel();
        DeviceManager{
            device_dao: DeviceDao::new(),
            config_map: HashMap::new(),
            upward_rx: upward_rx,
            upward_tx: upward_tx,
            downward_rx: None
        }
    }

    /// 添加单个设备
    pub fn add_device(&mut self) {
        
    }

    /// 开启双向线程
    /// - 设备管理 + 下行线程：从 mqtt 服务器接收到的消息，给设备的指令。下行传递线程也是设备操作的主线程，负责初始化并管理所有设备
    /// - 上行传递：接收从 device 来的消息，推送到 mqtt
    /// - 所有权关系：该函数将拿走 self 的所有权，因为需要在线程中调用访问 self 中的设备对象
    pub fn start_worker(self, downward_rx: mpsc::Receiver<DeviceCommandBo>, mqtt_client: Arc<MqttClient>, rt: &tokio::runtime::Runtime) {
        
        // 下行传递线程
        // - 向设备下达指令
        thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().expect("下行传递线程：无法创建 tokio 运行时");
            rt.block_on( async move {
                loop {
                    info!(LOG_TAG, "根据配置文件初始化设备");
                    let mut device_map: HashMap<String, Box<dyn Device + Sync + Send>> = init_devices_by_config_map(self.config_map.clone(), DeviceFactory::new());
    
                    info!(LOG_TAG, "等待下行指令");
                    let recv_message = downward_rx.recv();
                    match recv_message {
                        Ok(commnad) => {
                            let device_id = &commnad.device_id;
                            let device_ref = device_map.get(device_id).expect("获取设备失败");
                            if let Some(device_ref) = device_map.get_mut(device_id) {
                                info!(LOG_TAG, "下行指令：向设备 {} 发送指令：{:?}", device_id, commnad);
                                let _ = device_ref.cmd(commnad.action, commnad.params);
                            } else {
                                warn!(LOG_TAG, "下行指令：给设备发送指令失败，请求的设备 {} 不存在", device_id)
                            }
                        }
                        Err(e) => {
                            warn!(LOG_TAG, "下行指令通道关闭，即将退出，错误信息：{}", e);
                            return
                        }
                    }
                }
            });
        });
        
        info!(LOG_TAG, "下行指令 worker 已启动");

        let upward_rx = self.upward_rx;

        // 上行传递线程 （注意使用 tokio 调度）
        // - 设备上报数据
        // - 向 mqtt 发布推送设备状态
        thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().expect("上行传递线程：无法创建 tokio 运行时");
            rt.block_on( async move {
                loop{
                    info!(LOG_TAG, "等待上行数据");
                    let message = upward_rx.recv();
                    match message {
                        Ok(device_state_bo) => {
                            info!(LOG_TAG, "设备上报数据：{:?}", &device_state_bo);
                            mqtt_client.publish_status(device_state_bo).await.expect("向 mqtt 发布设备状态失败");
                        }   
                        Err(e) => {
                            warn!(LOG_TAG, "上行指令通道关闭，通道异常，错误信息：{}", e);
                            return
                        }
                    }
                }
            });
        });
        
        info!(LOG_TAG, "上行数据 worker 已启动");
    }

    pub fn clone_upward_tx(&self) -> mpsc::Sender<DeviceStateBo> {
        self.upward_tx.clone()
    }

    /// 系统初始化
    /// - 从远程配置文件中加载设备
    /// - 更新本地数据
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
            self.config_map.insert(device_config_po.device_id.clone(), device_config_po);
        }
        Ok(())
    }


    
}


/// 根据配置文件初始化设备
/// - 返回带有设备的 map
fn init_devices_by_config_map(config_map: HashMap<String, DevicePo>, device_factory: DeviceFactory) -> HashMap<String, Box<dyn Device + Sync + Send>> {
    let mut device_map: HashMap<String, Box<dyn Device + Sync + Send>> = HashMap::new();
        for (device_id, device_po) in config_map {
        let device_type = device_po.device_type.clone();
        let device_class = device_po.device_class.clone();
        match device_factory.create_device(device_po.clone()) {
            Ok(device) => {
                device_map.insert(device_id.clone(), device);
            }
            Err(e) => {
                warn!(LOG_TAG, "无法初始化设备，设备类型：{}，设备类别：{}，错误信息：{}", device_type, device_class, e);
            }
        }
    }
    device_map
}

// 将 json 转换为 po
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
    use crate::entity::bo::device_state_bo::DoControllerStateBo;
    use crate::mqtt_client::client::MqttClient;

    // 设备初始化测试
    #[test]
    fn test_device_create() {
        let _ = init_logger();
        let rt = tokio::runtime::Runtime::new().unwrap();
        let (tx, rx) = mpsc::channel();
        rt.block_on( async {
            let mut manager = DeviceManager::new();
            manager.config_map.insert("test_device_id".to_string(), DevicePo{
                device_id: "test_device_id".to_string(),
                device_class: "dummy".to_string(),
                device_type: "dummy".to_string(),
                name: "test_device_name".to_string(),
                description: "test_device_description".to_string(),
                room: "test_device_room".to_string(),
                config: "{}".to_string()
            });

            let mut mqtt_client = MqttClient::new();
            mqtt_client.start().await;
            let mqtt_client_arc = Arc::new(mqtt_client);

            manager.start_worker(rx, mqtt_client_arc.clone(), &rt);
        });
        tx.send(DeviceCommandBo{
            server_id: "this".to_string(),
            device_id: "test_device_id".to_string(),
            action: "some action".to_string(),
            params: serde_json::json!(null)
        }).unwrap();
        info!(LOG_TAG, "测试完成");
        thread::sleep(std::time::Duration::from_secs(6));
    }

    // 测试获取服务配置
    #[test] 
    fn test_get_device_config_from_remote() {
        let _ = init_logger();
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let mut manager = DeviceManager::new();
            manager.startup().await.unwrap();
        });
        info!(LOG_TAG, "测试完成");
    }

    // 下行传递测试
    #[test]
    fn test_downward_channel() {
        let _ = init_logger();
        println!("下行传递测试");
        let manager = DeviceManager::new();
        let (tx, rx) = mpsc::channel();
        let rt = tokio::runtime::Runtime::new().unwrap();

        rt.block_on(async {
            let mut mqtt_client = MqttClient::new();
            let _ = mqtt_client.start().await;
            let mqtt_client_arc = Arc::new(mqtt_client);
            manager.start_worker(rx, mqtt_client_arc.clone(), &rt);
        });

        thread::sleep(std::time::Duration::from_secs(1));

        println!("发送指令");
        tx.send(DeviceCommandBo{
            server_id: "this".to_string(),
            device_id: "123".to_string(),
            action: "test".to_string(),
            params: serde_json::json!(null)
        }).unwrap();

        // 等待 2 s
        // thread::sleep(std::time::Duration::from_secs(2));
        
        thread::sleep(std::time::Duration::from_secs(2));
    }

    // 上行传递测试
    #[test]
    fn test_upward_channel() {
        let _ = init_logger();
        println!("上行传递测试");
        let manager = DeviceManager::new();
        let (tx, rx) = mpsc::channel();
        let upward_tx = manager.clone_upward_tx();
        let rt = tokio::runtime::Runtime::new().unwrap();

        rt.block_on(async {
            let mut mqtt_client = MqttClient::new();
            let _ = mqtt_client.start().await;        
            let mqtt_client_arc = Arc::new(mqtt_client);
            manager.start_worker(rx, mqtt_client_arc.clone(), &rt);
        });

        let do_controller_bo = StateBoEnum::DoController(DoControllerStateBo{
            port: vec![1, 2, 3, 4]
        });

        upward_tx.send(DeviceStateBo{
            device_class: "test_class".to_string(),
            device_type: "test_type".to_string(),
            device_id: "123".to_string(),
            state: do_controller_bo
        }).unwrap();

        thread::sleep(std::time::Duration::from_secs(2));
    }
}