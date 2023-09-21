//! 设备配置缓存 dao 对象
use std::error::Error;
use rusqlite::params;
use rusqlite::Result;

use crate::common::sqlite::SqliteConnection;
use crate::entity::po::{FilePo, MediaTypeEnum};

struct FileDao {
    file_path: String,
    table_name: String
}

impl FileDao {
    pub fn new() -> Self {
        FileDao {
            file_path: String::from("cache/test.db"),
            table_name: String::from("file")
        }
    }

    /// 将单个文件信息加入缓存
    pub fn add_file_info(&self, file_info: FilePo) -> Result<()> {
        let conn = SqliteConnection::get().open()?;

        conn.execute(
            "INSERT INTO file (tag, orig_filename, filename, hash, media_type, deleted) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            (&file_info.tag, &file_info.orig_filename, &file_info.filename, &file_info.hash, file_info.media_type as u8, &file_info.deleted),
        )?;

        Ok(())
    }

    /// 获取所有的文件信息
    pub fn get_file(&self) -> Result<Vec<FilePo>> {
        let conn = SqliteConnection::get().open()?;

        let mut stmt = conn.prepare("SELECT tag, orig_filename, filename, hash, media_type, deleted FROM file")?;
        let file_iter = stmt.query_map([], |row| {
            Ok(FilePo {
                tag: row.get(0)?,
                orig_filename: row.get(1)?,
                filename: row.get(2)?,
                hash: row.get(3)?,
                media_type: match row.get(4)? {
                    1 => MediaTypeEnum::Audio,
                    2 => MediaTypeEnum::Video,
                    _ => MediaTypeEnum::Audio
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

    /// 清除缓存
    pub fn clear_cache(&self) -> Result<()> {
        self.recreate_table()?;
        Ok(())
    }

    /// 创建缓存数据表
    pub fn recreate_table(&self) -> Result<()> {
        let conn = SqliteConnection::get().open()?;

        conn.execute("DROP TABLE file", ())?;
        conn.execute(
            "CREATE TABLE file (
                id              INTEGER PRIMARY KEY autoincrement,
                tag             TEXT NOT NULL,
                orig_filename   TEXT NOT NULL,
                filename        TEXT NOT NULL,
                hash            TEXT NOT NULL,
                media_type      INTEGER NOT NULL,
                deleted         INTEGER NOT NULL
            )",
            (),
        )?;
        log::debug!("[Controller] 文件数据表初始化");

        Ok(())
    }
}