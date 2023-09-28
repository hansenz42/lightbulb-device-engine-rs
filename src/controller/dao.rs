use crate::common::sqlite::SqliteConnection;
use std::error::Error;
use async_trait::async_trait;

/// dao 特征

#[async_trait]
pub trait Dao {
    /// 检查表是否存在
    async fn check_table (&self, table_name: &'static str) -> Result<bool, Box<dyn Error>> {
        let conn = SqliteConnection::get().open().await?;

        let result = conn.call( move|conn| {
            let mut stmt = conn.prepare(format!("SELECT name FROM sqlite_master WHERE type='table' AND name='{}'", table_name).as_str())?;
            let table_iter = stmt.query_map([], |row| Ok(row.get::<usize, String>(0)?))?;
            
            let mut ret = false;
            for table in table_iter {
                ret = true;
                break;
            }

            Ok(ret)
        }).await?;
        Ok(result)
    }

    /// 创建数据表
    async fn create_table(&self) -> tokio_rusqlite::Result<()>;

    /// 删除数据表
    async fn drop_table(&self) -> tokio_rusqlite::Result<()>;

    /// 清空数据表
    fn clear_table(&self) -> tokio_rusqlite::Result<()> {
        self.drop_table();
        self.create_table();
        Ok(())
    }
}