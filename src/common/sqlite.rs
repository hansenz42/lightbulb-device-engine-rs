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

        Ok(())
    }

    pub fn get<'a>() -> &'a Self {
        // 给静态变量延迟赋值的宏
        lazy_static! {
            static ref CONN: SqliteConnection = SqliteConnection::new("cache/cache.db").expect("初始化 sqlite 连接失败");
        }
        &CONN
    }
}
