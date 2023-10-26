//! 文件管理器
//! 系统启动时
//! 1 从远程获取配置文件，如果获取到最新的设备配置，则保存到持久存储中
//! 2 如果无法读取，则使用本地缓存初始化
//! 3 如果不存在本地存储，则打断启动


use std::collections::HashMap;
use std::error::Error;
use serde_json::{Value, Map};

use crate::common::dao::Dao;
use super::file_dao::FileDao;
use crate::common::http;
use crate::entity::po::FilePo::FilePo;
use crate::entity::bo::FileBo::MediaTypeEnum;

const UPDATE_CONFIG_URL: &str = "/api/v1.2/file";

pub struct FileManager {
    file_dao: FileDao,
    cache: HashMap<String, FilePo>
}

impl FileManager {
    fn new() -> Self {
        FileManager {
            file_dao: FileDao::new(),
            cache: HashMap::new()
        }
    }

    /// 系统初始化
    async fn init(&self) -> Result<(), Box<dyn Error>> {
        // 保证数据表存在
        self.file_dao.ensure_table_exist().await?;
        Ok(())
    }

    /// 远程获取配置文件
    /// - 如果远程存在配置文件，那么一定是以远程文件为准
    /// - 如果远程文件读取失败，则使用本地文件缓存
    async fn get_file_config_from_remote(&mut self) -> Result<(), Box<dyn Error>> {
        // let json_data = http::api_get(UPDATE_CONFIG_URL).await?;
        match http::api_get(UPDATE_CONFIG_URL).await {
            Ok(json_data) => {
                // 清空已有数据，并保存当前数据
                self.file_dao.clear_table().await?;
                self.write_file_info_to_db(json_data).await?;
                self.load_file_info_from_db().await?;
                log::info!("[FileManager] 远程文件数据加载成功！")
            },
            Err(e) => {
                log::warn!("[FileManager] 无法获取远程配置文件，使用本地文件，错误信息：{}", e);
                self.load_file_info_from_db().await?;
            }
        }
        Ok(())
    }

    /// 将文件信息写入数据库
    async fn write_file_info_to_db(&self, json_data: Value) -> Result<(), Box<dyn Error>> {
        let file_list: &Vec<Value> = json_data.get("list").unwrap().as_array().expect("list 未找到");
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

    /// 从数据库读取数据当前对象
    async fn load_file_info_from_db(&mut self) -> Result<(), Box<dyn Error>> {
        let file_po_list: Vec<FilePo> = self.file_dao.get_all().await?;
        // 将 Vec 转换为 Hashmap 
        for file_po in file_po_list {
            self.cache.insert(file_po.hash.clone(), file_po);
        }
        Ok(())
    }

    /// 从远程下载文件到本地
    async fn download_file_from_remote(&self, file_po: &FilePo) -> Result<(), Box<dyn Error>> {
        let url = format!("/api/v1.1/file/{}", file_po.hash);
        let file_data = http::api_get(url.as_str()).await?;
        Ok(())
    }
    
}