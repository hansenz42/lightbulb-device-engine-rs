/// 数据库对象：设备
#[derive(Debug, Clone)]
pub struct DevicePo {
    // 设备二级类目
    pub device_class: String,
    // 设备类型
    pub device_type: String,
    // 设备名称
    pub name: String,
    // 设备描述
    pub description: String,
    // 设备区域（房间）
    pub room: String,
    // 设备配置（json string）
    pub config: String,
}