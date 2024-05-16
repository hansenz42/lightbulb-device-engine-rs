use serde_json::Value;

use crate::common::error::DriverError;

/// get type in json data, return string if ok, otherwise return error
pub fn get_str(json_data: &Value, value_name: &str) -> Result<String, DriverError>{
    let str = json_data[value_name].as_str().ok_or(DriverError("json parser: cannot find device_id in config".to_string()))?;
    Ok(str.to_string())
}

/// find config in json_data
pub fn get_config_str(config_data: &Value, value_name: &str) -> Result<String, DriverError>{
    let str = config_data[value_name].as_str().ok_or(DriverError(format!("json parser: cannot find {} in config", value_name)))?;
    Ok(str.to_string())
}

pub fn get_config_int(config_data: &Value, value_name: &str) -> Result<i64, DriverError>{
    config_data[value_name].as_i64().ok_or(DriverError(format!("json parser: cannot find {} in config", value_name)))
}