//! file system manager
//! control file metadata
//! manage local file cache

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use serde_json::{Value, Map};

use crate::common::dao::Dao;
use crate::common::error::{DeviceServerError, ServerErrorCode};
use super::file_dao::FileDao;
use super::file_repo::{FileRepo, FileMetaDto, FILE_FOLDER};
use crate::common::http;
use crate::entity::po::file_po::FilePo;
use crate::entity::dto::file_dto::MediaTypeEnum;
use lazy_static::lazy_static;
use crate::{info, warn, error, trace, debug};

const UPDATE_CONFIG_URL: &str = "api/v1.2/file/config";
const FILE_DOWNLOAD_URL: &str = "api/v1.2/file";
const LOG_TAG: &str = "FileManager";

pub struct FileController {
    file_dao: FileDao,
    file_repo: FileRepo,
    cache: Arc<Mutex<HashMap<String, FilePo>>>
}

impl FileController {
    pub fn new() -> Self {
        let mut obj = FileController {
            file_dao: FileDao::new(),
            file_repo: FileRepo::new(),
            cache: Arc::new(Mutex::new(HashMap::new()))
        };
        obj.update().expect("init file controller failed");
        obj
    }

    /// get singleton
    pub fn get() -> &'static Self {
        lazy_static! {
            static ref FILE_CONTROLLER: FileController = FileController::new();
        }
        &FILE_CONTROLLER
    }

    /// get remote file config files
    /// - if remote has config file, it must be according to remote
    /// - if remote read config file failed, use local cache
    async fn get_remote(&self) -> Result<Value, DeviceServerError> {
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
    async fn save_to_db(&self, map_to_write: HashMap<String, FilePo>) -> Result<(), DeviceServerError> {
        // clear local table
        self.file_dao.clear_table().await.map_err(
            |e| DeviceServerError {
                code: ServerErrorCode::DatabaseError,
                msg: format!("cannot save file config to db, clear table error: {}", e)
            }
        )?;
        for (_, file_po) in map_to_write.iter() {
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
    fn transform_list_to_po(&self, json_array: Value) -> Result<Vec<FilePo>, DeviceServerError> {
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

    /// read file config data from database
    async fn load_from_db(& self) -> Result<HashMap<String, FilePo>, DeviceServerError> {
        let mut ret = HashMap::new();
        let file_po_list: Vec<FilePo> = self.file_dao.get_all().await
            .map_err(|e| DeviceServerError {
                code: ServerErrorCode::DatabaseError,
                msg: format!("cannot read file meta from db, error: {}", e)
            })?;
        // 将 Vec 转换为 Hashmap 
        for file_po in file_po_list {
            ret.insert(file_po.hash.clone(), file_po);
        }
        Ok(ret)
    }

    /// download file from remote
    async fn download_file_from_remote(&self, file_po: &FilePo) -> Result<(), DeviceServerError> {
        let url = format!("{}/{}", FILE_DOWNLOAD_URL, file_po.hash);
        self.file_repo.download(url.as_str()).await
            .map_err(|e| DeviceServerError {
                code: ServerErrorCode::FileSystemError,
                msg: format!("download file error: {e}")
            })?;
        Ok(())
    }

    /// start the file manager
    /// 1 read local database 
    /// 2 scan files on the disk, check if the file exists, and update file meta
    /// 3 get remote file meta
    /// 4 check if there is new file on the remote server
    /// 5 download new file
    /// 6 save file config cache to db
    /// caution:
    /// - the file deleted on the remote, will not removed
    pub fn update(& self) -> Result<(), DeviceServerError> {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            // make sure file table exists
            self.file_dao.ensure_table_exist().await
                .map_err(|e| DeviceServerError {
                    code: ServerErrorCode::DatabaseError,
                    msg: format!("cannot ensure file table exist, error: {}", e)
                })?;

            // make local cache, load from database
            let mut local_file_map = self.load_from_db().await?;

            // scan disk, check if the file exists, and update file meta
            let scanned_file_list = self.file_repo.scan_files().await?;
            // vec 转换为 hashmap
            let scanned_file_hashmap: HashMap<String, FileMetaDto> = scanned_file_list.into_iter().map(|x| (x.hash.clone(), x.clone())).collect();

            for file in local_file_map.clone().values() {
                info!(LOG_TAG, "check cache record file name: {}; hash {}", file.filename, file.hash);
                if !scanned_file_hashmap.contains_key(&file.hash) {
                    // if file cannot find on the disk, delete database and cache record
                    self.file_dao.delete_file_info(&file.hash).await.map_err(|e| DeviceServerError {
                        code: ServerErrorCode::DatabaseError,
                        msg: format!("cannot delete file cache info data to db, error: {}", e)
                    })?;
                    local_file_map.remove(&file.hash);
                    warn!(LOG_TAG, "checking cache file on the disk, file does not exist on disk, deleted database record, name: {}, hash: {}", file.filename, file.hash);
                }
            }

            // get file meta from remote
            if let Ok(json_array) = self.get_remote().await {
                if let Ok(file_po_list) = self.transform_list_to_po(json_array.clone()) {
                    let mut new_file_po: Vec<FilePo> = Vec::new();
                    // compare with local cache ...
                    for file_po in &file_po_list {
                        if !local_file_map.contains_key(&file_po.hash) {
                            new_file_po.push(file_po.clone());
                            info!(LOG_TAG, "record new file {} to cache", file_po.filename);
                        }
                    }

                    // download new file
                    for file_po in &new_file_po {
                        match self.download_file_from_remote(file_po).await {
                            Ok(_) => {
                                local_file_map.insert(file_po.hash.clone(), file_po.clone());
                                info!(LOG_TAG, "downloaded file and record to cache, name: {}, hash: {}", file_po.filename, file_po.hash);
                            }
                            Err(e) => {
                                error!(LOG_TAG, "download file {} failed, error: {}", file_po.filename, e);
                            }
                        }
                    }

                    // replace object cache with new data
                    {
                        let mut map_guard = self.cache.lock().unwrap();
                        map_guard.clear();
                        map_guard.extend(local_file_map.clone());
                    }
                    // refresh database file meta with new data
                    self.save_to_db(local_file_map).await?;
                    info!(LOG_TAG, "file config cache successfully updated");
                } else {
                    warn!(LOG_TAG, "cannot transform json to file_po, data format error, will use local cache, parse failed data: {:?}", json_array);
                }
            } else {
                warn!(LOG_TAG, "cannot get remote data, use local cache");
            }
            Ok::<(), DeviceServerError>(())
        })
    }


    /// get file path by file hash
    pub fn get_path_by_hash(&self, hash_str: &str) -> Option<String> {
        {
            let map_guard = self.cache.lock().unwrap();
            let filename = map_guard.get(hash_str)?.filename.clone();
            return Some(format!("{}/{}", FILE_FOLDER, filename));
        }
    }
}

/// transform json object to file po
fn json_object_to_single_po(json_obj: &Value) -> Option<FilePo> {
    let file_data = json_obj.as_object().expect("file field incorrect");
    let file_po = FilePo {
        // tag can be empty
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
        let mut file_manager = FileController::new();
        file_manager.update().unwrap();
        let file_path = file_manager.get_path_by_hash("61b62be9d1715598003e71ec9ea52010");

        println!("{:?}", file_path);
    }

    #[test]
    fn test_get_data_from_flow() {
        let mut file_manager = FileController::new();
        let rt = tokio::runtime::Runtime::new().unwrap();

        rt.block_on(async {
            let json_data = file_manager.get_remote().await.unwrap();
            println!("{:?}", json_data);
        })
    }
}
