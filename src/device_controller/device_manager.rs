//! device manager implements two features
//! - manage device infomation: get data from flow server, store to local cache. if get remote data failed, use local cache
//! - multiple threads architecture, one thread for outgoing data, one thread for incoming data

use serde_json::{Map, Value};
use std::collections::HashMap;

use super::device_dao::DeviceDao;
use super::workers::device_thread::device_thread;
use super::workers::heartbeating_thread::heartbeating_thread;
use super::workers::reporting_thread::reporting_thread;
use super::entity::device_po::DevicePo;
use crate::common::dao::Dao;
use crate::common::error::{DeviceServerError, ServerErrorCode};
use crate::common::http;
use crate::device_controller::device_info_maker_helper::make_device_info;
use crate::entity::dto::device_command_dto::DeviceCommandDto;
use crate::entity::dto::device_meta_info_dto::DeviceMetaInfoDto;
use crate::entity::dto::device_state_dto::{DeviceStateDto, StateDtoEnum};
use crate::mqtt_client::client::MqttClient;
use crate::{debug, error, info, trace, warn};
use std::sync::{mpsc, Arc, Mutex};
use crate::common::setting::Settings;

// url to update device config
const UPDATE_CONFIG_URL: &str = "api/v1.2/device/config";
const LOG_TAG: &str = "device_manager";
const HEARTBEAT_INTERVAL: u64 = 10000;

/// device manager
/// - manage device list
/// - manage device incoming and outgoing data
/// - double thread architecture, one thread for outgoing data, one thread for incoming data
pub struct DeviceManager {
    device_dao: DeviceDao,
    // configuration map of devices
    pub config_map: HashMap<String, DevicePo>,
    // configuration list of devices
    pub config_list: Vec<DevicePo>,
    // downward receive channel from mqtt
    device_command_rx: mpsc::Receiver<DeviceCommandDto>,
    device_command_tx: mpsc::Sender<DeviceCommandDto>,
    // device info map
    pub device_info_map: Arc<Mutex<HashMap<String, DeviceMetaInfoDto>>>,
}

impl DeviceManager {
    pub fn new() -> Self {
        let (device_command_tx, device_command_rx) = mpsc::channel();
        DeviceManager {
            device_dao: DeviceDao::new(),
            config_map: HashMap::new(),
            config_list: Vec::new(),
            device_command_rx,
            device_command_tx,
            device_info_map: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn get_device_command_tx(&self) -> mpsc::Sender<DeviceCommandDto> {
        self.device_command_tx.clone()
    } 

    /// start the device manager work
    /// - heartbeating thread: send heartbeat periodically
    /// - device thread: create device and controller command sending
    /// - reporting thread: listen to devices status change and report to mqtt client
    /// 
    /// CAUTION: after calling this function, DeviceManager will drop, 
    /// so be sure that device_command_tx is cloned before calling this function
    pub fn run_threads(self, mqtt_client: Arc<Mutex<MqttClient>>) {
        let (state_report_tx, state_report_rx) = mpsc::channel();

        // 1 start device thread
        device_thread(
            state_report_tx,
            self.device_command_rx,
            self.config_list.clone(),
            self.device_info_map.clone(),
        );
        debug!(
            LOG_TAG,
            "device manager worker starting: device thread called"
        );

        // 2 start reporting thread
        reporting_thread(
            state_report_rx,
            mqtt_client.clone(),
            self.device_info_map.clone()
        );
        debug!(
            LOG_TAG,
            "device manager worker starting: reporting thread called"
        );

        // 3 start heartbeating thread
        heartbeating_thread(
            HEARTBEAT_INTERVAL,
            self.device_info_map.clone(),
            self.config_map.clone(),
            mqtt_client.clone(),
        );
        debug!(
            LOG_TAG,
            "device manager worker starting: heartbeating thread called"
        );
    }

    pub fn start(
        mut self,
        mqtt_client: Arc<Mutex<MqttClient>>,
    ) -> Result<(), DeviceServerError> {
        self.ready()?;
        self.run_threads(mqtt_client);
        Ok(())
    }

    /// init device manager
    /// - read config from remote server
    /// - update local data
    pub fn ready(&mut self) -> Result<(), DeviceServerError> {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
        // 1. make sure the table exists
            self.device_dao
                .ensure_table_exist()
                .await
                .map_err(|e| DeviceServerError {
                    code: ServerErrorCode::DatabaseError,
                    msg: format!("cannot ensure device table exist, error msg: {}", e),
                })?;

            // 2. get remote config data and write to db
            match self.get_remote().await {
                Ok(json_data) => {
                    // clear all data
                    self.device_dao
                        .clear_table()
                        .await
                        .map_err(|e| DeviceServerError {
                            code: ServerErrorCode::DatabaseError,
                            msg: format!("cannot save device config to db, clear table error: {}", e),
                        })?;
                    self.write_to_db(json_data).await?;
                    info!(
                        LOG_TAG,
                        "successfully got device config data from flow server"
                    );
                }
                Err(e) => {
                    error!(LOG_TAG, "cannot pull device config data from flow server, will use local data cache, err msg: {}", e);
                }
            }

            // 3. load data from database
            self.load_from_db().await?;
            info!(LOG_TAG, "successfully load device config data from db");

            // 4. make device info map
            self.device_info_map = Arc::new(Mutex::new(make_device_info(
                self.config_list.clone(),
            )?));

            Ok::<(), DeviceServerError>(())
        })?;

        info!(LOG_TAG, "device manager data ready");
        Ok(())
    }

    /// get config data from remote
    async fn get_remote(&mut self) -> Result<Value, DeviceServerError> {
        let url = format!("{}/{}", UPDATE_CONFIG_URL, Settings::get().server.server_id);
        info!(LOG_TAG, "get remote config data from flow server, url: {}", &url);
        http::api_get(url.as_str()).await
    }

    /// svae device config to db
    async fn write_to_db(&self, json_data: Value) -> Result<(), DeviceServerError> {
        let device_list = json_data
            .get("config")
            .unwrap()
            .as_array()
            .expect("error writing config, cannot find list in config");
        for device in device_list {
            if let Some(device_po) = transform_json_data_to_po(device.clone()) {
                self.device_dao
                    .add_device_config(device_po)
                    .await
                    .map_err(|e| DeviceServerError {
                        code: ServerErrorCode::DatabaseError,
                        msg: format!("error writing device config to database, error msg: {}", e),
                    })?;
            } else {
                warn!(LOG_TAG, "cannot parse device config json: {:?}", device);
            }
        }
        Ok(())
    }

    /// load config from database
    async fn load_from_db(&mut self) -> Result<(), DeviceServerError> {
        let device_config_po_list: Vec<DevicePo> =
            self.device_dao
                .get_all()
                .await
                .map_err(|e| DeviceServerError {
                    code: ServerErrorCode::DatabaseError,
                    msg: format!(
                        "error loading device config from database, error msg: {}",
                        e
                    ),
                })?;
        for device_config_po in device_config_po_list {
            self.config_map
                .insert(device_config_po.device_id.clone(), device_config_po.clone());
            self.config_list.push(device_config_po);
        }
        Ok(())
    }
}

// make json object to device po
fn transform_json_data_to_po(json_object: Value) -> Option<DevicePo> {
    let device_data = json_object.as_object()?;
    let device_po = DevicePo {
        device_id: json_object.get("device_id")?.as_str()?.to_string(),
        device_class: json_object.get("device_class")?.as_str()?.to_string(),
        device_type: json_object.get("device_type")?.as_str()?.to_string(),
        name: json_object.get("name")?.as_str()?.to_string(),
        description: json_object.get("description")?.as_str()?.to_string(),
        config: json_object.get("config")?.clone(),
    };
    Some(device_po)
}

// make device config to str
fn transform_device_config_obj_str(device_data: &Map<String, Value>) -> String {
    let mut config = device_data.clone();
    let config_str = serde_json::to_string(&config).unwrap();
    config_str
}

#[cfg(test)]
mod tests {
    use std::thread;

    use super::*;
    use crate::common::logger::init_logger;
    use crate::entity::dto::device_command_dto::CommandParamsEnum;
    use crate::mqtt_client::client::MqttClient;

    #[test]
    fn test_device_manager() {
        let _ = init_logger();
        let mut manager = DeviceManager::new();
        let tx = manager.get_device_command_tx();
        let mut mqtt_client = MqttClient::new();
        mqtt_client.start().unwrap();
        let mqtt_client_arc = Arc::new(Mutex::new(mqtt_client));
        manager.start(mqtt_client_arc).unwrap();

        // send command
        tx.send(DeviceCommandDto{
            device_id: "test_do_port".to_string(),
            server_id: "test".to_string(),
            action: "on".to_string(),
            params: CommandParamsEnum::Empty
        }).unwrap();

        // sleep 20 sec
        thread::sleep(std::time::Duration::from_secs(20));
        println!("test done");
    }

    /// test:
    /// 1, get data from flow server
    /// 2, load data to device manager
    /// 3, create devices
    /// 4, get device_info and device_enum
    #[test]
    fn test_device_engine_startup_and_make_devices() {
        let _ = init_logger();
        println!("device engine startup testing");
        let mut manager = DeviceManager::new();
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {})
    }
}
