use serde_json::Value;

use crate::common::error::DriverError;

/// get type in json data, return string if ok, otherwise return error
pub fn get_str(json_data: &Value, value_name: &str) -> Result<String, DriverError>{
    let str = json_data[value_name].as_str().ok_or(DriverError("json parser: cannot find device_id in config".to_string()))?;
    Ok(str.to_string())
}

/// find config in json_data
pub fn get_config_str(json_data: &Value, value_name: &str) -> Result<String, DriverError>{
    let config_data = json_data["config"].as_object().ok_or(DriverError("json parser: cannot find config in config".to_string()))?;
    let str = config_data[value_name].as_str().ok_or(DriverError("json parser: cannot find device_id in config".to_string()))?;
    Ok(str.to_string())
}

pub fn get_config_int(json_data: &Value, value_name: &str) -> Result<i64, DriverError>{
    let config_data = json_data["config"].as_object().ok_or(DriverError("json parser: cannot find config in config".to_string()))?;
    config_data[value_name].as_i64().ok_or(DriverError("json parser: cannot find device_id in config".to_string()))
}