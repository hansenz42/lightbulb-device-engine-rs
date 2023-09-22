//! 设备配置缓存 dao 对象
use rusqlite::params;
use rusqlite::Result;
use std::error::Error;

use crate::common::sqlite::SqliteConnection;
use crate::entity::po::{FilePo, MediaTypeEnum};

pub struct FileDao<'a> {
    file_path: &'a str,
    table_name: &'a str,
}

impl FileDao<'_> {
    pub fn new() -> Self {
        let obj = FileDao {
            file_path: "cache/test.db",
            table_name: "file",
        };
        let is_exist = obj.check_table().unwrap();
        if is_exist {
            log::debug!("文件数据表已存在");
        } else {
            obj.create_table().expect("创建文件数据表失败");
            log::debug!("文件数据表初始化");
        }
        return obj;
    }
    
    /// 检查表中是否存在 file 表
    pub fn check_table(&self) -> Result<bool> {
        let conn = SqliteConnection::get().open()?;

        let mut stmt = conn.prepare(
            format!(
                "SELECT name FROM sqlite_master WHERE type='table' AND name='{}'",
                self.table_name
            )
            .as_str(),
        )?;
        let file_iter = stmt.query_map([], |row| Ok(row.get::<usize, String>(0)?))?;

        let mut ret = false;
        for file in file_iter {
            ret = true;
            break;
        }

        Ok(ret)
    }

    /// 将单个文件信息加入缓存
    pub fn add_file_info(&self, file_info: FilePo) -> Result<()> {
        let conn = SqliteConnection::get().open()?;

        conn.execute(
            format!("INSERT INTO {} (tag, orig_filename, filename, hash, media_type, deleted) VALUES (?1, ?2, ?3, ?4, ?5, ?6)", self.table_name).as_str(),
            (&file_info.tag, &file_info.orig_filename, &file_info.filename, &file_info.hash, file_info.media_type as u8, &file_info.deleted),
        )?;

        Ok(())
    }

    /// 获取所有的文件信息
    pub fn get_file(&self) -> Result<Vec<FilePo>> {
        let conn = SqliteConnection::get().open()?;

        let mut stmt = conn.prepare(
            format!(
                "SELECT tag, orig_filename, filename, hash, media_type, deleted FROM {}",
                self.table_name
            )
            .as_str(),
        )?;
        let file_iter = stmt.query_map([], |row| {
            Ok(FilePo {
                tag: row.get(0)?,
                orig_filename: row.get(1)?,
                filename: row.get(2)?,
                hash: row.get(3)?,
                media_type: match row.get(4)? {
                    1 => MediaTypeEnum::Audio,
                    2 => MediaTypeEnum::Video,
                    _ => MediaTypeEnum::Audio,
                },
                deleted: row.get(5)?,
            })
        })?;

        let mut ret = Vec::new();
        for file in file_iter {
            ret.push(file?);
        }

        Ok(ret)
    }

    // 删除数据表
    fn drop_table(&self) -> Result<()> {
        let conn = SqliteConnection::get().open()?;

        conn.execute(
            format!("DROP TABLE {}", self.table_name).as_str(),
            (),
        )?;

        Ok(())
    }

    /// 创建缓存数据表
    fn create_table(&self) -> Result<()> {
        let conn = SqliteConnection::get().open()?;

        conn.execute(
            format!(
                "CREATE TABLE {} (
                id              INTEGER PRIMARY KEY autoincrement,
                tag             TEXT NOT NULL,
                orig_filename   TEXT NOT NULL,
                filename        TEXT NOT NULL,
                hash            TEXT NOT NULL,
                media_type      INTEGER NOT NULL,
                deleted         INTEGER NOT NULL
            )", self.table_name
            )
            .as_str(),
            (),
        )?;
        log::debug!("[Controller] 文件数据表初始化");

        Ok(())
    }
}
