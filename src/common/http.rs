//! networking module
//! function:
//! - 处理远程服务器错误：如果远程发生错误，code 不为 200，那么返回 HttpError
//! - 保证输入输出均为 json object

use std::{collections::HashMap, fs::File, io::Write};
use super::setting::Settings;
use lazy_static::lazy_static;
use serde_json::Value;
use std::error::Error;
use super::error::{DeviceServerError, ServerErrorCode};
use crate::{info, warn, error, trace, debug};

const LOG_TAG: &str = "Http-common-module";

lazy_static! {
    static ref SETTINGS: &'static Settings = Settings::get();
    static ref BASEURL: String = format!("http://{}:{}", SETTINGS.upstream.host, SETTINGS.upstream.port);
}

/// wrapper for get api
pub async fn api_get(api_url: &str) -> Result<Value, Box<dyn Error>> {
    let resp: serde_json::Value  = reqwest::get(format!("{}/{}", BASEURL.as_str(), api_url).as_str())
        .await?
        .json()
        .await?;
    let res = get_res_data(resp)?;
    Ok(res)
}

/// wrapper for post api
pub async fn api_post(api_url: &str, data: Value) -> Result<Value, Box<dyn Error>> {
    let resp: serde_json::Value = reqwest::Client::new()
        .post(format!("{}/{}", BASEURL.as_str(), api_url).as_str())
        .json(&data)
        .send()
        .await?
        .json()
        .await?;
    let res = get_res_data(resp)?;
    Ok(res)
}

pub async fn download_file(api_url: &str, folder_path: &str) -> Result<(), Box<dyn Error>> {
    let mut resp = reqwest::get(format!("{}/{}", BASEURL.as_str(), api_url)).await?;
    // 从 response Content-Disposition 中获取文件名
    let filename = resp
        .headers()
        .get("Content-Disposition")
        .and_then(|cd| cd.to_str().ok())
        .and_then(|cd| {
            cd.split(';')
                .find_map(|s| s.trim().strip_prefix("filename="))
                .map(|s| s.trim_matches('"'))
        })
        .unwrap_or("download.bin");
    // 检查是否存在同名文件
    let is_exist = tokio::fs::metadata(format!("{}/{}", folder_path, filename)).await.is_ok();
    if !is_exist {
        // 跳过该文件的保存
        let mut out = File::create(format!("{}/{}", folder_path, filename))?;
        while let Some(chunk) = resp.chunk().await? {
            out.write_all(&chunk)?;
        }
    } else {
        warn!(LOG_TAG, "文件 {} 已存在，跳过下载", filename);
    }
    
    Ok(())
}

/// 检查返回值中的 code 是否成功，不成功则抛出异常
fn get_res_data(resp: Value) -> Result<Value, Box<dyn Error>> {
    let status = resp["code"].as_i64().expect("status code not found");
    if status == 200 {
        Ok(resp["data"].clone())
    } else {
        Err(Box::new(DeviceServerError {code: ServerErrorCode::HttpError, msg: format!("http 请求错误: {}", resp["msg"].as_str().unwrap_or(""))}))
    }
}