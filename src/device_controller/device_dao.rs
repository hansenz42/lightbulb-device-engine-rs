//! 设备配置缓存 dao 对象
use crate::common::dao::Dao;
use std::error::Error;
use std::result::Result;
use rusqlite::params;

use crate::common::sqlite::SqliteConnection;
use super::entity::device_po::DevicePo;
use async_trait::async_trait;
use crate::{debug, error, info, trace, warn};

pub struct DeviceDao {
    file_path: &'static str,
    table_name: &'static str,
}

const LOG_TAG: &str = "device_dao";

#[async_trait]
impl Dao for DeviceDao {
    async fn drop_table(&self) -> tokio_rusqlite::Result<()> {
        let conn = SqliteConnection::get().open().await?;
        let table_name_copy = self.table_name;

        conn.call( move|conn| 
            conn.execute(format!("DROP TABLE {}", table_name_copy).as_str(), ())
        ).await?;
        Ok(())
    }

    async fn create_table(&self) -> tokio_rusqlite::Result<()> {
        let conn = SqliteConnection::get().open().await?;

        conn.call(|conn| {
            conn.execute(
                "CREATE TABLE device (
                        id              INTEGER PRIMARY KEY autoincrement,
                        device_id       TEXT NOT NULL,
                        device_class    TEXT NOT NULL,
                        device_type     TEXT NOT NULL,
                        name            TEXT NOT NULL,
                        description     TEXT NOT NULL,
                        config          TEXT NOT NULL
                    )",
                (),
            )
        })
        .await?;

        debug!(LOG_TAG, "device cache init complete");

        Ok(())
    }
}

impl DeviceDao {
    pub fn new() -> Self {
        DeviceDao {
            file_path: "cache/test.db",
            table_name: "device",
        }
    }

    pub async fn ensure_table_exist(&self) -> Result<(), Box<dyn Error>> {
        let is_exist = self.check_table(self.table_name).await?;
        if is_exist {
            debug!(LOG_TAG, "device cache table already exist");
        } else {
            self.create_table().await?;
            debug!(LOG_TAG, "device cache table init");
        }
        Ok(())
    }    

    /// 将单个设备加入缓存
    pub async fn add_device_config(&self, device_config: DevicePo) -> tokio_rusqlite::Result<()> {
        let conn = SqliteConnection::get().open().await?;

        let device_config_copy = device_config.clone();

        conn.call(move |conn| {
            conn.execute(
                "INSERT INTO device (device_id, device_class, device_type, name, description, config) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    device_config_copy.device_id,
                    device_config_copy.device_class,
                    device_config_copy.device_type, 
                    device_config_copy.name, 
                    device_config_copy.description,
                    device_config_copy.config.to_string()
                ],
            )
        }).await?;

        Ok(())
    }

    pub async fn get_all(&self) -> tokio_rusqlite::Result<Vec<DevicePo>> {
        let conn = SqliteConnection::get().open().await?;

        let res = conn.call(|conn| {
            let mut stmt = conn.prepare(
                "SELECT device_id, device_class, device_type, name, description, config FROM device",
            )?;
            let device_iter = stmt.query_map([], |row| {
                let config_str: String = row.get(5)?;
                Ok(DevicePo {
                    device_id: row.get(0)?,
                    device_class: row.get(1)?,
                    device_type: row.get(2)?,
                    name: row.get(3)?,
                    description: row.get(4)?,
                    config: serde_json::from_str(&config_str).unwrap_or_default(),
                })
            })?;
    
            let mut ret = Vec::new();
            for device in device_iter {
                ret.push(device?);
            }
    
            Ok(ret)
        }).await?;

        Ok(res)
    }
}
