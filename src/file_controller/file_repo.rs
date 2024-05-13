//! 文件仓库，控制文件读写
use std::error::Error;
use std::io::{BufReader, Read, Write};
use std::fs::File;
use crypto::digest::Digest;
use crypto::md5::Md5;
use async_trait::async_trait;
use data_encoding::HEXUPPER;
use crate::common::error::{DeviceServerError, ServerErrorCode};
use crate::common::http::download_file;

pub const FILE_FOLDER: &'static str = "file";

/// 计算 md5 hash
fn md5_digest<R: Read>(mut reader: R) -> Result<String, Box<dyn Error>> {
    let mut md5 = Md5::new();
    let mut buffer = [0; 1024];

    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        md5.input(&buffer[..count]);
    }

    Ok(md5.result_str())
}

/// 本地文件描述数据
#[derive(Debug, Clone)]
pub struct FileMetaDto {
    pub filename: String,
    pub hash: String
}

/// 管理本地文件存储
/// - 保存文件
/// - 读取当前文件夹下所有文件的 hash
pub struct FileRepo{}


impl FileRepo {
    pub fn new() -> Self {
        FileRepo{}
    }

    /// make sure "files" folder exists
    pub async fn check_folder() -> Result<(), DeviceServerError> {
        // 检查 folder 是否存在
        let is_exist = tokio::fs::metadata(FILE_FOLDER).await.is_ok();
        if !is_exist {
            // 创建文件夹
            tokio::fs::create_dir(FILE_FOLDER).await
                .map_err(|e| DeviceServerError {code: ServerErrorCode::FileSystemError, msg: format!("create dir error: {e}")})?;
        }
        Ok(())
    }

    /// download file and save to local cache
    pub async fn download(&self, url: &str) -> Result<(), DeviceServerError>{
        download_file(url, FILE_FOLDER).await
            .map_err(|e| DeviceServerError {code: ServerErrorCode::FileSystemError, msg: format!("download file error: {e}")})?;
        Ok(())
    }

    /// calculate md5 according to file
    pub fn hash_file(&self, filename: &str) -> Result< String, DeviceServerError>{
        // 打开文件并计算哈希
        let input = File::open(format!("{}/{}", FILE_FOLDER, filename))
            .map_err(|e| DeviceServerError {code: ServerErrorCode::FileSystemError, msg: format!("open file error: {e}")})?;
        let reader = BufReader::new(input);
        let result_str = md5_digest(reader).map_err(|e| DeviceServerError {code: ServerErrorCode::FileSystemError, msg: format!("hash file error: {e}")})?;
        Ok(result_str)
    }

    /// 检查本地目录下的所有文件
    pub async fn scan_files(&self) -> Result<Vec<FileMetaDto>, DeviceServerError> {
        let mut file_list = Vec::new();
        // check File_FOLDER exists, if not, then create one
        let is_exist = tokio::fs::metadata(FILE_FOLDER).await.is_ok();
        if !is_exist {
            tokio::fs::create_dir(FILE_FOLDER).await
                .map_err(|e| DeviceServerError {code: ServerErrorCode::FileSystemError, msg: format!("create dir error: {e}")})?;
        }
        let mut dir = tokio::fs::read_dir(FILE_FOLDER).await
            .map_err(|e| DeviceServerError {code: ServerErrorCode::FileSystemError, msg: format!("read dir error: {e}")})?;
        while let Some(res) = dir.next_entry().await
            .map_err(|e| DeviceServerError {code: ServerErrorCode::FileSystemError, msg: format!("dir to next entry error: {e}")})? 
            {
                let file_name = res.file_name().into_string().unwrap();
                let hash = FileRepo::new().hash_file(&file_name)?;
                file_list.push(FileMetaDto {
                    filename: file_name,
                    hash
                });
            }
        Ok(file_list)
    }
}


#[cfg(test)]
mod test {
    use super::*;
    use crate::common::logger::{init_logger};

    /// 测试单文件哈希
    #[test]
    fn test_md5_hash() {
        init_logger();
        let file_repo = FileRepo::new();
        let rt = tokio::runtime::Runtime::new().unwrap();

        rt.block_on(async {
            FileRepo::check_folder().await.expect("检查文件夹失败");
            // file_repo.download("http://localhost:7001/api/v1.1/file/1bd20cf9ad0fb02f00764a6434b59a64").await.expect("下载文件失败");
            let hash = file_repo.hash_file("142504926885316_2.wav").expect("计算文件 hash 失败");
            println!("hash: {}", hash);
        });
    }

    // 测试文件目录哈希
    #[test]
    fn test_scan_files() {
        init_logger();
        let rt = tokio::runtime::Runtime::new().unwrap();

        rt.block_on(async {
            let file_repo = FileRepo::new();
            FileRepo::check_folder().await.expect("检查文件夹失败");
            let file_list = file_repo.scan_files().await.expect("扫描文件失败");
            println!("file_list: {:?}", file_list);
        });
    }
}