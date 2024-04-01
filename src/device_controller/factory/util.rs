use serde_json::Value;

use crate::common::error::DriverError;

/// check type in json_data, return string if ok, otherwise return error
/// device config json must have "device_type"
pub fn check_type(json_data: &Value, valid_type: &str) -> Result<String, DriverError>{
    let device_type = json_data["device_type"].as_str().ok_or(DriverError("device factory: cannot find device_type in config".to_string()))?;
    if device_type!= valid_type {
        return Err(DriverError("device factory: device_type must be dmx_bus".to_string()));
    }
    Ok(device_type.to_string())
}

/// get type in json data, return string if ok, otherwise return error
pub fn get_str(json_data: &Value, value_name: &str) -> Result<String, DriverError>{
    let str = json_data[value_name].as_str().ok_or(DriverError("device factory: cannot find device_id in config".to_string()))?;
    Ok(str.to_string())
}

/// find config in json_data
pub fn get_config_str(json_data: &Value, value_name: &str) -> Result<String, DriverError>{
    let config_data = json_data["config"].as_object().ok_or(DriverError("device factory: cannot find config in config".to_string()))?;
    let str = config_data[value_name].as_str().ok_or(DriverError("device factory: cannot find device_id in config".to_string()))?;
    Ok(str.to_string())
}

pub fn get_config_int(json_data: &Value, value_name: &str) -> Result<i64, DriverError>{
    let config_data = json_data["config"].as_object().ok_or(DriverError("device factory: cannot find config in config".to_string()))?;
    config_data[value_name].as_i64().ok_or(DriverError("device factory: cannot find device_id in config".to_string()))
}