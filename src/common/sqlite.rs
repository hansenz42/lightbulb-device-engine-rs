use tokio_rusqlite::{Connection};
use lazy_static::lazy_static;
use log;
use std::error::Error;

pub struct SqliteConnection {
    // sqlite 文件路径
    file_name: String,
}

impl SqliteConnection {
    pub fn new(file_name: &str) -> Result<Self, Box<dyn Error>> {
        let conn = SqliteConnection {
            file_name: file_name.to_string(),
        };
        conn.check_folder()?;
        Ok(conn)
    }

    fn check_folder(&self) -> Result<(), Box<dyn Error>> {
        let path = std::path::Path::new(&self.file_name);
        let parent = path.parent().unwrap();
        if !parent.exists() {
            log::info!("创建文件夹: {:?}", parent);
            std::fs::create_dir_all(parent)?;
        }
        Ok(())
    } 

    pub async fn open(&self) -> tokio_rusqlite::Result<Connection> {
        Ok(Connection::open(&self.file_name).await?)
    }

    pub fn get<'a>() -> &'a Self {
        // 给静态变量延迟赋值的宏
        lazy_static! {
            static ref CONN: SqliteConnection = SqliteConnection::new("cache/cache.db").expect("初始化 sqlite 连接失败");
        }
        &CONN
    }
}
