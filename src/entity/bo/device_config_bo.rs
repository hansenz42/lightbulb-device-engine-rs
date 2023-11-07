//! 设备配置 Bo

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
    DmxCustom(DmxCustomConfigBo)
}

#[derive(Debug, Clone)]
pub struct DeviceConfigBo {
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
pub struct ModbusConfigBo {
    pub serial_port: String,
    pub baudrate: i32
}

#[derive(Debug, Clone)]
pub struct DmxBusConfigBo {
    pub ftdi_serial: String,
}

#[derive(Debug, Clone)]
pub struct SerialBusConfigBo {
    pub serial_port: String,
    pub baudrate: i32
}

#[derive(Debug, Clone)]
pub struct ModbusDigitalInputControllerConfigBo {
    pub unit: i32,
    pub input_num: i32,
}

#[derive(Debug, Clone)]
pub struct ModbusDigitalOutputControllerConfigBo {
    pub unit: i32,
    pub output_num: i32,
}

#[derive(Debug, Clone)]
pub enum AudioChannelEnum {
    Left,
    Right,
    Stereo
}

#[derive(Debug, Clone)]
pub struct AudioConfigBo {
    pub soundcard_id: String,
    pub to_channel: AudioChannelEnum
}

#[derive(Debug, Clone)]
pub struct DiConfigBo {
    pub address:i32
}

#[derive(Debug, Clone)]
pub struct DoConfigBo {
    pub address:i32
}

#[derive(Debug, Clone)]
pub struct RemoteConfigBo {
    pub num_button: i32
}

#[derive(Debug, Clone)]
pub struct DmxCustomConfigBo {
    pub channels: i32,
    pub address: i32
}