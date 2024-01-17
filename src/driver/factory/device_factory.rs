use super::super::traits::device::Device;
use std::collections::HashMap;
use super::traits::Factory;
use crate::{
    entity::{bo::device_config_bo::{DeviceCreateBo, ConfigBo}, po::device_po::DevicePo}, 
    common::error::{DeviceServerError, ServerErrorCode}
};
use crate::driver::device::device_enum::DeviceEnum;

use super::{
    modbus_factory::ModbusFactory,
    dummy_factory::DummyFactory,
};


pub struct DeviceFactory{
    factory_map: HashMap<String, Box<dyn Factory>>,
}

impl DeviceFactory {
    // 实例化工厂对象，同时也实例化所有的工厂类
    pub fn new() -> Self {
        let mut factory_map: HashMap<String, Box<dyn Factory>> = HashMap::new();
        factory_map.insert("modbus".to_string(), Box::new(ModbusFactory::new()));
        factory_map.insert("dummy".to_string(), Box::new(DummyFactory::new()));
        // factory_map.insert("serial".to_string(), Box::new(SerialFactory::new()));
        DeviceFactory{
            factory_map: factory_map
        }
    }

    /// 从 DevicePo 创建设备
    pub fn create_device(&self, device_po: DevicePo) -> Result<Box<dyn Device + Sync + Send>, DeviceServerError> {
        let device_type = device_po.device_type.clone();
        let factory = self.factory_map.get(&device_type).ok_or(DeviceServerError{
            code: ServerErrorCode::DeviceTypeNotSupport,
            msg: format!("不支持的设备类型：{}", device_type)
        })?;
        Ok(factory.create(device_po)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::bo::device_config_bo::DummyConfigBo;

    #[test]
    fn test_create_dummy() {
        let device_po = DevicePo{
            device_id: "test_device_id".to_string(),
            device_class: "test_device_class".to_string(),
            device_type: "dummy".to_string(),
            name: "".to_string(),
            room: "".to_string(),
            description: "".to_string(),
            config: "{}".to_string()
        };
        let device_factory = DeviceFactory::new();
        let device = device_factory.create_device(device_po).unwrap();
        assert_eq!(device.get_category().1, "dummy");
        println!("已生成设备 dummy");
    }
}