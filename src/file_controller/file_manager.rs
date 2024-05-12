//! 文件管理器
//! 系统启动时
//! 1 从远程获取配置文件，如果获取到最新的设备配置，则保存到持久存储中
//! 2 如果无法读取，则使用本地缓存初始化
//! 3 如果不存在本地存储，则打断启动


use std::collections::HashMap;
use std::error::Error;
use std::fmt::format;
use serde_json::{Value, Map};

use crate::common::dao::Dao;
use crate::common::error::{DeviceServerError, ServerErrorCode};
use super::file_dao::FileDao;
use super::file_repo::{FileRepo, FileMetaBo};
use crate::common::http;
use crate::entity::po::file_po::FilePo;
use crate::entity::dto::file_dto::MediaTypeEnum;
use crate::{info, warn, error, trace, debug};

const UPDATE_CONFIG_URL: &str = "api/v1.2/file/config";
const LOG_TAG: &str = "FileManager";

pub struct FileManager {
    file_dao: FileDao,
    file_repo: FileRepo,
    cache: HashMap<String, FilePo>
}

impl FileManager {
    pub fn new() -> Self {
        FileManager {
            file_dao: FileDao::new(),
            file_repo: FileRepo::new(),
            cache: HashMap::new()
        }
    }

    /// get remote file config files
    /// - if remote has config file, it must be according to remote
    /// - if remote read config file failed, use local cache
    async fn get_remote(&mut self) -> Result<Value, DeviceServerError> {
        let response_data = http::api_get(UPDATE_CONFIG_URL).await;
        match response_data {
            Ok(json_data) => {
                if json_data.is_array() {
                    info!(LOG_TAG, "successfully get remote file data");
                    Ok(json_data.clone())
                } else {
                    error!(LOG_TAG, "remote file data format error");
                    return Err(DeviceServerError {
                        code: ServerErrorCode::FileConfigError, 
                        msg: format!("remote file data format error") }
                    );
                }
            }
            Err(e) => {
                error!(LOG_TAG, "cannot get remote file config data: {}", e);
                Err(e)
            }
        }
    }

    /// save local cache to db
    async fn save_to_db(&self) -> Result<(), DeviceServerError> {
        // 清空本地数据库
        self.file_dao.clear_table().await.map_err(
            |e| DeviceServerError {
                code: ServerErrorCode::DatabaseError,
                msg: format!("cannot save file config to db, clear table error: {}", e)
            }
        )?;
        for (_, file_po) in self.cache.iter() {
            self.file_dao.add_file_info(file_po.clone()).await.map_err(
                |e| DeviceServerError {
                    code: ServerErrorCode::DatabaseError,
                    msg: format!("cannot add file cache info data to db, error: {}", e)
                }
            )?;
        }
        Ok(())
    }

    /// make json_array as po
    async fn transform_list_to_po(&self, json_array: Value) -> Result<Vec<FilePo>, DeviceServerError> {
        let mut file_po_list: Vec<FilePo> = Vec::new();
        if let Some(file_list) = json_array.as_array() {
            for file in file_list {
                if let Some(file_po) = json_object_to_single_po(file) {
                    file_po_list.push(file_po);
                } else {
                    warn!(LOG_TAG, "single file data format error, parse failed data: {:?}", file);
                }
            }
            Ok(file_po_list)
        } else {
            error!(LOG_TAG, "cannot transform json to file_po, data format error, parse failed data: {:?}", json_array);
            return Err(DeviceServerError {
                code: ServerErrorCode::FileConfigError,
                msg: format!("cannot transform json to file_po, data format error, parse failed data: {:?}", json_array)
            })
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
    pub async fn startup(&mut self) -> Result<(), DeviceServerError> {
        // 确保 file 数据表存在
        self.file_dao.ensure_table_exist().await
            .map_err(|e| DeviceServerError {
                code: ServerErrorCode::DatabaseError,
                msg: format!("cannot ensure file table exist, error: {}", e)
            })?;

        // 从数据库生成本地缓存
        match self.load_from_db().await {
            Ok(_) => {
                info!(LOG_TAG, "read file config cache from db success");
            }
            Err(e) => {
                warn!(LOG_TAG, "read file config cache from db failed, error: {}", e);
            }
        }

        // 扫描磁盘，检查文件是否都存在，如果文件不存在，则更新本地缓存的数据（删除文件记录）
        let scanned_file_list = self.file_repo.scan_files().await?;
        // vec 转换为 hashmap
        let scanned_file_hashmap: HashMap<String, FileMetaBo> = scanned_file_list.into_iter().map(|x| (x.hash.clone(), x.clone())).collect();

        for file in self.cache.clone().values() {
            info!(LOG_TAG, "check cache record file name: {}; hash {}", file.filename, file.hash);
            if !scanned_file_hashmap.contains_key(&file.hash) {
                // 删除数据库中的记录
                self.file_dao.delete_file_info(&file.hash).await.map_err(|e| DeviceServerError {
                    code: ServerErrorCode::DatabaseError,
                    msg: format!("cannot delete file cache info data to db, error: {}", e)
                })?;
                // 删除本地记录
                self.cache.remove(&file.hash);
                warn!(LOG_TAG, "check cache file on the disk, file does not exist on disk, deleted database and cache record, name: {}, hash: {}", file.filename, file.hash);
            }
        }

        // 获取远程文件记录
        if let Ok(json_array) = self.get_remote().await {
            if let Ok(file_po_list) = self.transform_list_to_po(json_array.clone()).await {
                let mut new_file_po: Vec<FilePo> = Vec::new();
                // 将远程数据和本地数据进行比较
                for file_po in &file_po_list {
                    if !self.cache.contains_key(&file_po.hash) {
                        new_file_po.push(file_po.clone());
                        info!(LOG_TAG, "record new file {} to cache", file_po.filename);
                    }
                }

                // 下载新文件
                for file_po in &new_file_po {
                    match self.download_file_from_remote(file_po).await {
                        Ok(_) => {
                            self.cache.insert(file_po.hash.clone(), file_po.clone());
                            info!(LOG_TAG, "downloaded file and record to cache, name: {}, hash: {}", file_po.filename, file_po.hash);
                        }
                        Err(e) => {
                            error!(LOG_TAG, "download file {} failed, error: {}", file_po.filename, e);
                        }
                    }
                }

                // 更新当前缓存
                self.save_to_db().await?;
                info!(LOG_TAG, "file config cache successfully updated");
            } else {
                warn!(LOG_TAG, "cannot transform json to file_po, data format error, will use local cache, parse failed data: {:?}", json_array);
            }
        } else {
            warn!(LOG_TAG, "cannot get remote data, use local cache");
        }

        Ok(())
    }
    
}

/// 辅助函数：将单个 json object 转换为 FilePo，如果转换失败，则返回 None
fn json_object_to_single_po(json_obj: &Value) -> Option<FilePo> {
    let file_data = json_obj.as_object().expect("file field incorrect");
    let file_po = FilePo {
        // tag 有可能为空
        tag: file_data.get("tag").expect("get tag from file_data").as_str().or_else(|| Some(""))?.to_string(),
        filename: file_data.get("filename")?.as_str()?.to_string(),
        hash: file_data.get("hash")?.as_str()?.to_string(),
        media_type: match file_data.get("type")?.as_str()? {
            "audio" => MediaTypeEnum::Audio,
            "video" => MediaTypeEnum::Video,
            _ => panic!("media_type field incorrect")
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

    #[test]
    fn test_get_data_from_flow() {
        let mut file_manager = FileManager::new();
        let rt = tokio::runtime::Runtime::new().unwrap();

        rt.block_on(async {
            let json_data = file_manager.get_remote().await.unwrap();
            println!("{:?}", json_data);
        })
    }
}
