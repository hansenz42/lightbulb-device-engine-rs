//! 网络请求模块
//! 功能：
//! - 处理远程服务器错误：如果远程发生错误，code 不为 200，那么返回 HttpError
//! - 保证输入输出均为 json object

use std::collections::HashMap;
use super::setting::Settings;
use lazy_static::lazy_static;
use serde_json::Value;
use std::error::Error;
use super::error::{DeviceServerError, ErrorCode};

lazy_static! {
    static ref SETTINGS: &'static Settings = Settings::get();
    static ref BASEURL: String = format!("http://{}:{}", SETTINGS.upstream.host, SETTINGS.upstream.port);
}

/// GET 方法请求 api
pub async fn api_get(api_url: &str) -> Result<Value, Box<dyn Error>> {
    let resp: serde_json::Value  = reqwest::get(format!("{}/{}", BASEURL.as_str(), api_url).as_str())
        .await?
        .json()
        .await?;
    let res = get_res_data(resp)?;
    Ok(res)
}

/// POST 方法请求 api
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

/// 检查返回值中的 code 是否成功，不成功则抛出异常
fn get_res_data(resp: Value) -> Result<Value, Box<dyn Error>> {
    let status = resp["code"].as_i64().expect("status code not found");
    if status == 200 {
        Ok(resp["data"].clone())
    } else {
        Err(Box::new(DeviceServerError {code: ErrorCode::HttpError, msg: format!("http 请求错误: {}", resp["msg"].as_str().unwrap_or(""))}))
    }
}