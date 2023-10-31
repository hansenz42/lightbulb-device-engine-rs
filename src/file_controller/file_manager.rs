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
use super::file_repo::{FileRepo, FileMetaBo};
use crate::common::http;
use crate::entity::po::FilePo::FilePo;
use crate::entity::bo::FileBo::MediaTypeEnum;
use crate::{info, warn, error, trace, debug};

const UPDATE_CONFIG_URL: &str = "/api/v1.2/file";
const LOG_TAG: &str = "FileManager";

pub struct FileManager {
    file_dao: FileDao,
    file_repo: FileRepo,
    cache: HashMap<String, FilePo>
}

impl FileManager {
    fn new() -> Self {
        FileManager {
            file_dao: FileDao::new(),
            file_repo: FileRepo::new(),
            cache: HashMap::new()
        }
    }

    /// 远程获取配置文件
    /// - 如果远程存在配置文件，那么一定是以远程文件为准
    /// - 如果远程文件读取失败，则使用本地文件缓存
    async fn get_remote(&mut self) -> Result<Value, Box<dyn Error>> {
        let response_data = http::api_get(UPDATE_CONFIG_URL).await;
        match response_data {
            Ok(json_data) => {
                if let Some(list) = json_data.get("list") {
                    if list.is_array() {
                        info!(LOG_TAG, "成功获取远程文件数据");
                        Ok(list.clone())
                    } else {
                        error!(LOG_TAG, "远程文件数据格式错误");
                        return Err("远程文件数据格式错误".into());
                    }
                } else {
                    error!(LOG_TAG, "远程文件数据格式错误");
                    return Err("远程文件数据格式错误".into());
                }
            }
            Err(e) => {
                error!(LOG_TAG, "无法获取远程文件数据，错误信息：{}", e);
                Err(e)
            }
        }
    }

    /// 将本地的缓存更新到数据库
    async fn save_to_db(&self) -> Result<(), Box<dyn Error>> {
        // 清空本地数据库
        self.file_dao.clear_table().await?;
        for (_, file_po) in self.cache.iter() {
            self.file_dao.add_file_info(file_po.clone()).await?;
        }
        Ok(())
    }

    /// 将远程返回的 json_array 数组转换为可处理的 po
    async fn transform_list_to_po(&self, json_array: Value) -> Result<Vec<FilePo>, Box<dyn Error>> {
        let mut file_po_list: Vec<FilePo> = Vec::new();
        if let Some(file_list) = json_array.as_array() {
            for file in file_list {
                if let Some(file_po) = json_object_to_single_po(file) {
                    file_po_list.push(file_po);
                } else {
                    warn!(LOG_TAG, "单条文件数据格式错误，解析失败 data: {:?}", file);
                }
            }
            Ok(file_po_list)
        } else {
            error!(LOG_TAG, "数据转换错误，无法将接收的文件信息转换为数据对对象");
            return Err("数据转换错误，无法将接收的文件信息转换为数据对对象".into());
        }
    }

    /// 从数据库读取数据当前对象
    async fn load_from_db(&mut self) -> Result<(), Box<dyn Error>> {
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
        self.file_repo.download(url.as_str()).await?;
        Ok(())
    }

    /// 启动流程
    /// - 从数据库生成本地缓存
    /// - 扫描磁盘，检查文件是否都存在，如果文件不存在，则更新本地缓存的数据（删除文件记录）
    /// - 获取远程文件记录
    /// - 检查远程数据和本地文件的不同
    /// - （如有文件未下载）则下载文件
    /// - 将最新的数据写入缓存，保存到数据库
    /// 注意：目前本地旧的文件不会被清理
    async fn startup(&mut self) -> Result<(), Box<dyn Error>> {
        // 确保 file 数据表存在
        self.file_dao.ensure_table_exist().await?;

        // 从数据库生成本地缓存
        match self.load_from_db().await {
            Ok(_) => {
                info!(LOG_TAG, "从数据库读取数据成功");
            }
            Err(e) => {
                warn!(LOG_TAG, "本地缓读取失败，缓存不存在，错误信息：{}", e);
            }
        }

        // 扫描磁盘，检查文件是否都存在，如果文件不存在，则更新本地缓存的数据（删除文件记录）
        let scanned_file_list = self.file_repo.scan_files().await?;
        // vec 转换为 hashmap
        let scanned_file_hashmap: HashMap<String, FileMetaBo> = scanned_file_list.into_iter().map(|x| (x.hash.clone(), x.clone())).collect();

        for file in self.cache.clone().values() {
            info!(LOG_TAG, "检查缓存记录中的文件 name: {}; hash {}", file.filename, file.hash);
            if !scanned_file_hashmap.contains_key(&file.hash) {
                // 删除数据库中的记录
                self.file_dao.delete_file_info(&file.hash).await?;
                // 删除本地记录
                self.cache.remove(&file.hash);
                warn!(LOG_TAG, "文件在磁盘上不存在，已删除数据库和缓存记录， name {}, hash {}", file.filename, file.hash);
            }
        }

        // 获取远程文件记录
        if let Ok(json_array) = self.get_remote().await {
            if let Ok(file_po_list) = self.transform_list_to_po(json_array).await {
                let mut new_file_po: Vec<FilePo> = Vec::new();
                // 将远程数据和本地数据进行比较
                for file_po in &file_po_list {
                    if !self.cache.contains_key(&file_po.hash) {
                        new_file_po.push(file_po.clone());
                        info!(LOG_TAG, "记录新文件 {} 到程序缓存", file_po.filename);
                    }
                }

                // 下载新文件
                for file_po in &new_file_po {
                    match self.download_file_from_remote(file_po).await {
                        Ok(_) => {
                            self.cache.insert(file_po.hash.clone(), file_po.clone());
                            info!(LOG_TAG, "已下载文件并记录到缓存 name {} ; hash {}", file_po.filename, file_po.hash);
                        }
                        Err(e) => {
                            error!(LOG_TAG, "下载文件 {} 失败，错误信息：{}", file_po.filename, e);
                        }
                    }
                }

                // 更新当前缓存
                self.save_to_db().await?;
                info!(LOG_TAG, "缓存更新成功，数据库更新成功，文件管理器已启动");
            } else {
                warn!(LOG_TAG, "转成数据转换为本地失败，使用本地缓存");
            }
        } else {
            warn!(LOG_TAG, "无法获取远程数据，使用本地缓存");
        }

        Ok(())
    }
    
}

/// 辅助函数：将单个 json object 转换为 FilePo，如果转换失败，则返回 None
fn json_object_to_single_po(json_obj: &Value) -> Option<FilePo> {
    let file_data = json_obj.as_object().expect("file 数据格式错误");
    let file_po = FilePo {
        // tag 有可能为空
        tag: file_data.get("tag").expect("get tag from file_data").as_str().or_else(|| Some(""))?.to_string(),
        filename: file_data.get("filename")?.as_str()?.to_string(),
        hash: file_data.get("hash")?.as_str()?.to_string(),
        media_type: match file_data.get("type")?.as_str()? {
            "audio" => MediaTypeEnum::Audio,
            "video" => MediaTypeEnum::Video,
            _ => panic!("media_type 字段数据错误")
        },
        // delete 字段将数据库中的 int 转换为 bool
        deleted: file_data.get("deleted")?.as_bool()?
    };
    Some(file_po)
}


#[cfg(test)]
mod test {
    use super::*;
    use crate::common::logger::{init_logger};

    #[test]
    fn test() {
        init_logger();
        let mut file_manager = FileManager::new();
        let rt = tokio::runtime::Runtime::new().unwrap();

        rt.block_on(async {
            file_manager.startup().await.unwrap();
        });
    }
}
