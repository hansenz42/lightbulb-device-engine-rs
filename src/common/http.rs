//! networking module
//! function:
//! - if return code is not 200, return HttpError
//! - make input and output are json format

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
pub async fn api_get(api_url: &str) -> Result<Value, DeviceServerError> {
    let resp: serde_json::Value  = reqwest::get(format!("{}/{}", BASEURL.as_str(), api_url).as_str())
        .await.map_err(|e| DeviceServerError {code: ServerErrorCode::HttpError, msg: format!("http GET error: {e}")})?
        .json()
        .await.map_err(|e| DeviceServerError {code: ServerErrorCode::HttpError, msg: format!("http GET error, cannot transform return value to json: {e}")})?;
    let res = get_res_data(resp)?;
    Ok(res)
}

/// wrapper for post api
pub async fn api_post(api_url: &str, data: Value) -> Result<Value, DeviceServerError> {
    let resp: serde_json::Value = reqwest::Client::new()
        .post(format!("{}/{}", BASEURL.as_str(), api_url).as_str())
        .json(&data)
        .send()
        .await.map_err(|e| DeviceServerError {code: ServerErrorCode::HttpError, msg: format!("http POST error: {e}")})?
        .json()
        .await.map_err(|e| DeviceServerError {code: ServerErrorCode::HttpError, msg: format!("http POST error, cannot transform return value to json: {e}")})?;
    let res = get_res_data(resp)?;
    Ok(res)
}

pub async fn download_file(api_url: &str, folder_path: &str) -> Result<(), DeviceServerError> {
    let mut resp = reqwest::get(format!("{}/{}", BASEURL.as_str(), api_url)).await
        .map_err(|e| DeviceServerError {code: ServerErrorCode::HttpError, msg: format!("file downloading error, http GET error: {e}")})?;
    // get filename from response Content-Disposition
    let filename = resp
        .headers()
        .get("Content-Disposition")
        .and_then(|cd| cd.to_str().ok())
        .and_then(|cd| {
            cd.split(';')
                .find_map(|s| s.trim().strip_prefix("filename="))
                .map(|s| s.trim_matches('"'))
        })
        .unwrap_or("download-default-name.bin");
    // check if the file exists with the same name
    let is_exist = tokio::fs::metadata(format!("{}/{}", folder_path, filename)).await.is_ok();
    if !is_exist {
        // skip saving file
        let mut out = File::create(format!("{}/{}", folder_path, filename)).map_err(|e| DeviceServerError {code: ServerErrorCode::FileSystemError, msg: format!("file downloading error, fail to create file: {e}")})?;
        while let Some(chunk) = resp.chunk().await.map_err(|e| DeviceServerError {code: ServerErrorCode::FileSystemError, msg: format!("file downloading error, http get error: {e}")})? {
            out.write_all(&chunk).map_err(|e| DeviceServerError {code: ServerErrorCode::FileSystemError, msg: format!("file downloading error, fail to write file: {e}")})?;
        }
    } else {
        warn!(LOG_TAG, "file {} exists, download skippped", filename);
    }
    
    Ok(())
}

/// check if the return code is 200
fn get_res_data(resp: Value) -> Result<Value, DeviceServerError> {
    let status = resp["code"].as_i64().expect("status code not found");
    if status == 200 {
        Ok(resp["data"].clone())
    } else {
        Err(DeviceServerError {code: ServerErrorCode::HttpError, msg: format!("http error: {}", resp["msg"].as_str().unwrap_or(""))})
    }
}