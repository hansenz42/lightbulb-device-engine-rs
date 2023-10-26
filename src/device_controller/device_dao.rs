//! 设备配置缓存 dao 对象
use crate::common::dao::Dao;
use std::error::Error;
use std::result::Result;
use rusqlite::params;

use crate::common::sqlite::SqliteConnection;
use crate::entity::po::DevicePo::DevicePo;
use async_trait::async_trait;

pub struct DeviceDao {
    file_path: &'static str,
    table_name: &'static str,
}

#[async_trait]
impl Dao for DeviceDao {
    /// 删除数据表
    async fn drop_table(&self) -> tokio_rusqlite::Result<()> {
        let conn = SqliteConnection::get().open().await?;
        let table_name_copy = self.table_name.clone();

        // 执行删除
        conn.call( move|conn| 
            conn.execute(format!("DROP TABLE {}", table_name_copy).as_str(), ())
        ).await?;
        Ok(())
    }

    /// 创建数据表
    async fn create_table(&self) -> tokio_rusqlite::Result<()> {
        let conn = SqliteConnection::get().open().await?;

        conn.call(|conn| {
            conn.execute(
                "CREATE TABLE device (
                        id              INTEGER PRIMARY KEY autoincrement,
                        device_class    TEXT NOT NULL,
                        device_type     TEXT NOT NULL,
                        name            TEXT NOT NULL,
                        description     TEXT NOT NULL,
                        room            TEXT NOT NULL,
                        config          TEXT NOT NULL
                    )",
                (),
            )
        })
        .await?;

        log::debug!("[Controller] 设备缓存表初始化");

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
            log::debug!("设备缓存表已存在");
        } else {
            self.create_table().await?;
            log::debug!("设备缓存表初始化");
        }
        Ok(())
    }    

    /// 将单个设备加入缓存
    pub async fn add_device_config(&self, device_config: DevicePo) -> tokio_rusqlite::Result<()> {
        let conn = SqliteConnection::get().open().await?;

        let device_config_copy = device_config.clone();

        conn.call(move |conn| {
            conn.execute(
                "INSERT INTO device (device_class, device_type, name, description, room, config) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![device_config_copy.device_class, device_config_copy.device_type, device_config_copy.name, device_config_copy.description, device_config_copy.room, device_config_copy.config],
            )
        }).await?;

        Ok(())
    }

    /// 读取整个列表
    pub async fn get_device_config_list(&self) -> tokio_rusqlite::Result<Vec<DevicePo>> {
        let conn = SqliteConnection::get().open().await?;

        let res = conn.call(|conn| {
            let mut stmt = conn.prepare(
                "SELECT device_class, device_type, name, description, room, config FROM device",
            )?;
            let device_iter = stmt.query_map([], |row| {
                Ok(DevicePo {
                    device_class: row.get(0)?,
                    device_type: row.get(1)?,
                    name: row.get(2)?,
                    description: row.get(3)?,
                    room: row.get(4)?,
                    config: row.get(5)?,
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
