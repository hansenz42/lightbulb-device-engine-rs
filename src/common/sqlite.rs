use rusqlite::{Connection, Result};
use lazy_static::lazy_static;
use log;

pub struct SqliteConnection {
    // sqlite 文件路径
    file_name: String,
}

impl SqliteConnection {
    pub fn new(file_name: &str) -> Result<Self> {
        let ret = SqliteConnection {
            file_name: file_name.to_string(),
        };
        ret.init_tables()?;
        Ok(ret)
    }

    pub fn open(&self) -> Result<Connection> {
        Connection::open(&self.file_name)
    }

    /// 创建数据表
    fn init_tables(&self) -> Result<()> {
        let conn = self.open()?;
        conn.execute(
            "CREATE TABLE file (
                id              INTEGER PRIMARY KEY,
                tag             TEXT NOT NULL,
                orig_filename   TEXT NOT NULL,
                filename        TEXT NOT NULL,
                hash            TEXT NOT NULL,
                media_type      INTEGER NOT NULL,
                deleted         INTEGER NOT NULL
            )",
            (),
        )?;
        conn.execute(
            "CREATE TABLE device (
                id              INTEGER PRIMARY KEY,
                device_class    TEXT NOT NULL,
                device_type     TEXT NOT NULL,
                name            TEXT NOT NULL,
                description     TEXT NOT NULL,
                room            TEXT NOT NULL,
                config          TEXT NOT NULL
            )",
            (),
        )?;
        log::info!("sqlite 缓存数据表初始化完成");
        Ok(())
    }
}
