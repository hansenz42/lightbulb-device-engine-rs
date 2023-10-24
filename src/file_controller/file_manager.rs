//! 文件管理器
use std::error::Error;
use serde_json::{Value, Map};

use crate::common::dao::Dao;
use super::file_dao::FileDao;
use crate::common::http;
use crate::entity::po::{FilePo, MediaTypeEnum};

const UPDATE_CONFIG_URL: &str = "/api/v1.2/file";

struct FileManager {
    file_dao: FileDao
}

impl FileManager {
    fn new() -> Self {
        FileManager {
            file_dao: FileDao::new()
        }
    }

    /// 系统初始化
    async fn init(&self) -> Result<(), Box<dyn Error>> {
        // 保证数据表存在
        self.file_dao.ensure_table_exist().await?;
        Ok(())
    }

    /// 获取文件配置
    async fn get_file_config_from_remote(&self) -> Result<(), Box<dyn Error>> {
        let result = http::api_get(UPDATE_CONFIG_URL).await?;
        self.file_dao.clear_table().await?;
        Ok(())
    }

    /// 将远程文件写入数据库
    async fn write_file_config_to_local_cache(&self, json_data: Value) -> Result<(), Box<dyn Error>> {
        let file_list = json_data.get("list").unwrap().as_array().expect("list 未找到");
        for file in file_list {
            let file_data = file.as_object().expect("file 数据格式错误");
            let file_po = FilePo {
                tag: file_data.get("tag").unwrap().as_str().unwrap().to_string(),
                filename: file_data.get("filename").unwrap().as_str().unwrap().to_string(),
                hash: file_data.get("hash").unwrap().as_str().unwrap().to_string(),
                media_type: match file_data.get("media_type").unwrap().as_str().unwrap() {
                    "audio" => MediaTypeEnum::Audio,
                    "video" => MediaTypeEnum::Video,
                    _ => panic!("media_type 字段数据错误")
                },
                // delete 字段将数据库中的 int 转换为 bool
                deleted: match file_data.get("deleted").unwrap().as_u64().unwrap() {
                    0 => false,
                    1 => true,
                    _ => panic!("deleted 字段数据错误")
                },
            };
            self.file_dao.add_file_info(file_po).await?;
        }
        Ok(())
    }
}