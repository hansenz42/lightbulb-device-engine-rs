//! 设备管理器，主要实现两部分功能
//! - 设备配置信息管理：从远程服务器获得配置数据，并保存到本地缓存。优先下载远程数据，如果远程数据下载失败，则使用本地缓存
//! - 设备通信管理：双线程结构，一个负责上行通信，一个负责下行通信
//! - 下行通信线程管理设备实例，避免了实例在多线程传递的问题。下行通信线程也是 device_manager 的主线程，用于写不同的子设备
//! - 上行通信线程另一边为 Device 中的 runner 设备，runner 设备会轮询接口并在特定时间向上发送数据
//! - 此外，设备管理器还应该维护一系列有 runner 特征的设备，这些设备可以挂载子设备，但是需要运行独立的线程


use std::cell::RefCell;
use std::collections::HashMap;
use std::error::Error;
use serde_json::{Value, Map};

use crate::common::dao::Dao;
use crate::common::error::{DeviceServerError, ServerErrorCode};
use crate::driver::dmx::dmx_bus::DmxBus;
use crate::driver::serial::serial_bus::SerialBus;
use crate::driver::traits::Operable;
use crate::entity::bo::device_config_bo::{ConfigBo};
use crate::mqtt_client::client::MqttClient;
use super::device_dao::DeviceDao;
use crate::common::http;
use super::entity::device_po::DevicePo;
use crate::entity::bo::device_state_bo::{DeviceStateBo, StateBoEnum};
use crate::entity::bo::device_command_bo::DeviceCommandBo;
use crate::{info, warn, error, trace, debug};
use std::thread;
use std::sync::{mpsc, Arc, Mutex};
use crate::driver::modbus::modbus_bus::ModbusBus;

// url to update device config
const UPDATE_CONFIG_URL: &str = "api/v1.2/device/config/HZ-B1";
const LOG_TAG : &str = "DeviceManager";

/// device manager
/// - manage device list
/// - manage device incoming and outgoing data
/// - double thread architecture, one thread for outgoing data, one thread for incoming data
pub struct DeviceManager {
    device_dao: DeviceDao,
    // configuration map of devices
    pub config_map: HashMap<String, DevicePo>,
    
    // upward thread: receive from device, send to mqtt
    upward_rx: mpsc::Receiver<DeviceStateBo>,
    
    // the device can clone this rx channel to send data to upward thread
    upward_rx_dummy: mpsc::Sender<DeviceStateBo>,
    
    // downward receive channel from mqtt
    downward_rx: Option<mpsc::Receiver<DeviceCommandBo>>,
}

impl DeviceManager {
    pub fn new() -> Self {
        let (upward_rx_dummy, upward_rx) = mpsc::channel();
        DeviceManager{
            device_dao: DeviceDao::new(),
            config_map: HashMap::new(),
            upward_rx: upward_rx,
            upward_rx_dummy: upward_rx_dummy,
            downward_rx: None,
        }
    }

    /// start bidirectional communication
    /// - 设备管理 + 下行线程：从 mqtt 服务器接收到的消息，给设备的指令。下行传递线程也是设备操作的主线程，负责初始化并管理所有设备
    /// - 上行传递：接收从 device 来的消息，推送到 mqtt
    /// - 所有权关系：该函数将拿走 self 的所有权，因为需要在线程中调用访问 self 中的设备对象
    pub fn start_worker(self, downward_rx: mpsc::Receiver<DeviceCommandBo>, mqtt_client: Arc<MqttClient>) {
        
        info!(LOG_TAG, "init devices according to config");


        // downward thread, send command to device
        thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().expect("downward worker: cannot create tokio runtime");
            rt.block_on( async move {
                loop {
                    info!(LOG_TAG, "waitting for downward command");
                    let recv_message = downward_rx.recv();
                    match recv_message {
                        Ok(commnad) => {
                            let device_id = &commnad.device_id;

                        }
                        Err(e) => {
                            warn!(LOG_TAG, "downward worker channel closing, error msg: {}", e);
                            return
                        }
                    }
                }
            });
        });
        
        info!(LOG_TAG, "download worker started");

        let upward_rx = self.upward_rx;

        // 上行传递线程 （注意使用 tokio 调度）
        // - 设备上报数据
        // - 向 mqtt 发布推送设备状态
        thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().expect("upward worker: cannot create tokio runtime");
            rt.block_on( async move {
                loop{
                    info!(LOG_TAG, "waiting for upward message");
                    let message = upward_rx.recv();
                    match message {
                        Ok(device_state_bo) => {
                            info!(LOG_TAG, "upward message to mqtt: {:?}", &device_state_bo);
                            mqtt_client.publish_status(device_state_bo).await.expect("cannot publish upward message to mqtt");
                        }   
                        Err(e) => {
                            warn!(LOG_TAG, "upward worker closed, channel error, msg: {}", e);
                            return
                        }
                    }
                }
            });
        });
        
        info!(LOG_TAG, "upward worker started");
    }

    pub fn clone_upward_tx(&self) -> mpsc::Sender<DeviceStateBo> {
        self.upward_rx_dummy.clone()
    }

    /// init device manager
    /// - read config from remote server
    /// - update local data
    pub async fn startup(&mut self) -> Result<(), Box<dyn Error>> {
        self.device_dao.ensure_table_exist().await?;

        match self.get_remote().await {
            Ok(json_data) => {
                // clear all data
                self.device_dao.clear_table().await?;
                self.write_to_db(json_data).await?;
                info!(LOG_TAG, "远程设备配置加载成功！");
            }
            Err(e) => {
                warn!(LOG_TAG, "无法获取远程设备配置文件，将使用本地缓存配置文件，错误信息：{}", e);
            }
        }

        self.load_from_db().await?;
        info!(LOG_TAG, "device manager initialized");
        Ok(())
    }

    /// get config data from remote
    async fn get_remote(&mut self) -> Result<Value, DeviceServerError>{
        http::api_get(UPDATE_CONFIG_URL).await
    }

    /// svae device config to db
    async fn write_to_db(&self, json_data: Value) -> Result<(), DeviceServerError>{
        let device_list = json_data.get("list").unwrap().as_array().expect("error writing config, cannot find list in config");
        for device in device_list {
            if let Some(device_po) = transform_json_data_to_po(device.clone()) {
                self.device_dao.add_device_config(device_po).await
                .map_err(|e| DeviceServerError{
                    code: ServerErrorCode::DatabaseError,
                    msg: format!("error writing device config to database, error msg: {}", e)
                })?;
            } else {
                warn!(LOG_TAG, "cannot parse device config json: {:?}", device);
            }
        }
        Ok(())
    }

    /// load config from database
    async fn load_from_db(&mut self) -> Result<(), Box<dyn Error>> {
        let device_config_po_list: Vec<DevicePo> = self.device_dao.get_all().await?;
        for device_config_po in device_config_po_list {
            self.config_map.insert(device_config_po.device_id.clone(), device_config_po);
        }
        Ok(())
    }


    
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

            manager.start_worker(rx, mqtt_client_arc.clone());
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
            manager.start_worker(rx, mqtt_client_arc.clone());
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
        let upward_rx_dummy = manager.clone_upward_tx();
        let rt = tokio::runtime::Runtime::new().unwrap();

        rt.block_on(async {
            let mut mqtt_client = MqttClient::new();
            let _ = mqtt_client.start().await;        
            let mqtt_client_arc = Arc::new(mqtt_client);
            manager.start_worker(rx, mqtt_client_arc.clone());
        });

        let do_controller_bo = StateBoEnum::DoController(DoControllerStateBo{
            port: vec![1, 2, 3, 4]
        });

        upward_rx_dummy.send(DeviceStateBo{
            device_class: "test_class".to_string(),
            device_type: "test_type".to_string(),
            device_id: "123".to_string(),
            state: do_controller_bo
        }).unwrap();

        thread::sleep(std::time::Duration::from_secs(2));
    }

    #[test]
    fn test_get_device_config() {
        let _ = init_logger();
        println!("get device config testing");
        let mut manager = DeviceManager::new();
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let result = manager.get_remote().await.unwrap();
            println!("{:?}", result);
        })
    }
}