//! 设备配置和创建设备相关 Bo

use serde::{Deserialize, Serialize};


#[derive(Debug, Clone)]
pub struct DeviceCreateBo {
    // 设备一级类目
    pub device_class: String,
    // 设备二级类目
    pub device_type: String,
    // 设备 id
    pub device_id: String,
    // 设备名称
    pub name: String,
    // 设备描述
    pub description: String,
    // 设备区域（房间）
    pub room: String,
    // 设备配置（json string）
    pub config: ConfigBo,

    // 主设备 id，只有子设备存在才有效
    pub master_device_id: Option<String>
}

#[derive(Debug, Clone)]
pub enum ConfigBo {
    Modbus(ModbusConfigBo),
    DmxBus(DmxBusConfigBo),
    SerialBus(SerialBusConfigBo),
    ModbusDigitalInputController(ModbusDigitalInputControllerConfigBo),
    ModbusDigitalOutputController(ModbusDigitalOutputControllerConfigBo),
    Audio(AudioConfigBo),
    Di(DiConfigBo),
    Do(DoConfigBo),
    Remote(RemoteConfigBo),
    DmxCustom(DmxCustomConfigBo),
    Dummy(DummyConfigBo)
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DummyConfigBo {

}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModbusConfigBo {
    pub serial_port: String,
    pub baudrate: u32
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DmxBusConfigBo {
    pub ftdi_serial: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SerialBusConfigBo {
    pub serial_port: String,
    pub baudrate:u32
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModbusDigitalInputControllerConfigBo {
    pub unit: u32,
    pub input_num: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModbusDigitalOutputControllerConfigBo {
    pub unit: u32,
    pub output_num: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum AudioChannelEnum {
    Left,
    Right,
    Stereo
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AudioConfigBo {
    pub soundcard_id: String,
    pub to_channel: AudioChannelEnum
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DiConfigBo {
    pub address:u32
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DoConfigBo {
    pub address:u32
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RemoteConfigBo {
    pub num_button: u32
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DmxCustomConfigBo {
    pub channels: u32,
    pub address: u32
}