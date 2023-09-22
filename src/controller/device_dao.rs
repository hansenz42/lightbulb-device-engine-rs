//! 设备配置缓存 dao 对象
use std::error::Error;
use rusqlite::params;
use rusqlite::Result;

use crate::common::sqlite::SqliteConnection;
use crate::entity::po::DevicePo;

pub struct DeviceDao {
    file_path: String,
    table_name: String
}

impl DeviceDao {
    pub fn new() -> Self {
        let obj = DeviceDao {
            file_path: String::from("cache/test.db"),
            table_name: String::from("device")
        };
        let is_exist = obj.check_table().unwrap();
        if is_exist {
            log::debug!("设备缓存表已存在");
        } else {
            obj.create_table().expect("创建设备缓存表失败");
            log::debug!("设备缓存表初始化");
        }
        return obj;
    }

    pub fn check_table (&self) -> Result<bool> {
        let conn = SqliteConnection::get().open()?;

        let mut stmt = conn.prepare(format!("SELECT name FROM sqlite_master WHERE type='table' AND name='{}'", self.table_name).as_str())?;
        let device_iter = stmt.query_map([], |row| Ok(row.get::<usize, String>(0)?))?;

        let mut ret = false;
        for device in device_iter {
            ret = true;
            break;
        }

        Ok(ret)
    }

    /// 将单个设备加入缓存
    pub fn add_device_config(&self, device_config: DevicePo) -> Result<()> {
        let conn = SqliteConnection::get().open()?;

        conn.execute(
            "INSERT INTO device (device_class, device_type, name, description, room, config) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![device_config.device_class, device_config.device_type, device_config.name, device_config.description, device_config.room, device_config.config],
        )?;

        Ok(())
    }

    /// 读取整个列表
    pub fn get_device_config_list(&self) -> Result<Vec<DevicePo>> {
        let conn = SqliteConnection::get().open()?;

        let mut stmt = conn.prepare("SELECT device_class, device_type, name, description, room, config FROM device")?;
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
    }

    /// 删除数据表
    fn drop_table(&self) -> Result<()> {
        let conn = SqliteConnection::get().open()?;

        conn.execute(format!("DROP TABLE {}", self.table_name).as_str(), ())?;

        Ok(())
    }

    /// 创建数据表
    fn create_table(&self) -> Result<()> {
        let conn = SqliteConnection::get().open()?;

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
        )?;
        log::debug!("[Controller] 设备缓存表初始化");

        Ok(())
    }
}