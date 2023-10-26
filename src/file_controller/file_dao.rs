//! 设备配置缓存 dao 对象
use rusqlite::params;
use std::error::Error;
use crate::common::dao::Dao;

use crate::common::sqlite::SqliteConnection;
use crate::entity::po::FilePo::FilePo;
use crate::entity::bo::FileBo::MediaTypeEnum;
use async_trait::async_trait;

pub struct FileDao {
    file_path: &'static str,
    table_name: &'static str,
}

#[async_trait]
impl Dao for FileDao {
    /// 删除数据表
    async fn drop_table(&self) -> tokio_rusqlite::Result<()> {
        let conn = SqliteConnection::get().open().await?;
        let table_name_copy = self.table_name.clone();

        conn.call(move |conn| {
            conn.execute(
                format!("DROP TABLE {}", table_name_copy).as_str(),
                (),
            )
        }).await?;

        Ok(())
    }

    /// 创建缓存数据表
    async fn create_table(&self) -> tokio_rusqlite::Result<()> {
        let conn = SqliteConnection::get().open().await?;
        let table_name_copy = self.table_name.clone();

        conn.call( move |conn| {
            conn.execute(
                format!(
                    "CREATE TABLE {} (
                    id              INTEGER PRIMARY KEY autoincrement,
                    tag             TEXT NOT NULL,
                    filename        TEXT NOT NULL,
                    hash            TEXT NOT NULL,
                    media_type      INTEGER NOT NULL,
                    deleted         INTEGER NOT NULL
                )", table_name_copy
                )
                .as_str(),
                (),
            )
        }).await?;

        
        log::debug!("[Controller] 文件数据表初始化");

        Ok(())
    }
}

impl FileDao {
    pub fn new() -> Self {
        FileDao {
            file_path: "cache/test.db",
            table_name: "file",
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
    

    /// 将单个文件信息加入缓存
    pub async fn add_file_info(&self, file_info: FilePo) -> Result<(), Box<dyn Error>> {
        let conn = SqliteConnection::get().open().await?;

        let file_info_copy = file_info.clone();
        let table_name  = self.table_name.clone();

        conn.call(move |conn| {
            conn.execute(
                format!("INSERT INTO {} (tag, filename, hash, media_type, deleted) VALUES (?1, ?2, ?3, ?4, ?5, ?6)", table_name).as_str(),
                (file_info_copy.tag, file_info_copy.filename, file_info_copy.hash, file_info_copy.media_type as u8, file_info_copy.deleted),
            )
        }).await?;

        Ok(())
    }

    /// 获取所有的文件信息
    pub async fn get_all(&self) -> Result<Vec<FilePo>, Box<dyn Error>> {
        let conn = SqliteConnection::get().open().await?;

        let table_name_copy = self.table_name.clone();

        let file_pos = conn.call(move |conn| {
            let mut stmt = conn.prepare(
                format!("SELECT tag, filename, hash, media_type, deleted FROM {}", table_name_copy).as_str(),
            )?;

            let files = stmt.query_map([], |row| {
                Ok(FilePo {
                    tag: row.get(0)?,
                    filename: row.get(1)?,
                    hash: row.get(2)?,
                    media_type: match row.get(3)? {
                        1 => MediaTypeEnum::Audio,
                        2 => MediaTypeEnum::Video,
                        _ => MediaTypeEnum::Audio,
                    },
                    deleted: row.get(4)?,
                })
            })?
            .collect::<Result<Vec<FilePo>, rusqlite::Error>>()?;
            Ok(files)
        }).await?;


        let mut ret = Vec::new();
        for file_po in file_pos {
            ret.push(file_po);
        }

        Ok(ret)
    }
}
