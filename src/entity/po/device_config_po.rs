//! 设备配置和创建设备相关 Bo

use serde::{Deserialize, Serialize};

/// used for creating device
#[derive(Debug, Clone)]
pub struct DeviceCreatePo {
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
    pub config: ConfigPo,

    // 主设备 id，只有子设备存在才有效
    pub master_device_id: Option<String>
}

#[derive(Debug, Clone)]
pub enum ConfigPo {
    Modbus(ModbusConfigPo),
    DmxBus(DmxBusConfigPo),
    SerialBus(SerialBusConfigPo),
    ModbusDigitalInputController(ModbusDigitalInputControllerConfigPo),
    ModbusDigitalOutputController(ModbusDigitalOutputControllerConfigPo),
    Audio(AudioConfigPo),
    Di(DiConfigPo),
    Do(DoConfigPo),
    Remote(RemoteConfigPo),
    DmxCustom(DmxCustomConfigPo),
    Dummy(DummyConfigPo)
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DummyConfigPo {

}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModbusConfigPo {
    pub serial_port: String,
    pub baudrate: u32
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DmxBusConfigPo {
    pub ftdi_serial: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SerialBusConfigPo {
    pub serial_port: String,
    pub baudrate:u32
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModbusDigitalInputControllerConfigPo {
    pub unit: u32,
    pub input_num: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModbusDigitalOutputControllerConfigPo {
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
pub struct AudioConfigPo {
    pub soundcard_id: String,
    pub to_channel: AudioChannelEnum
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DiConfigPo {
    pub address:u32
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DoConfigPo {
    pub address:u32
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RemoteConfigPo {
    pub num_button: u32
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DmxCustomConfigPo {
    pub channels: u32,
    pub address: u32
}