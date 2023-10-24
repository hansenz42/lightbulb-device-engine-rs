//! 数据库实体类
//! 文件数据实体
//! 设备数据实体

#[derive(Debug, Clone)]
pub enum MediaTypeEnum {
    Audio = 1,
    Video = 2
}

/// 数据库对象：文件
#[derive(Debug, Clone)]
pub struct FilePo {
    // 标签
    pub tag: String,
    // 服务器上的文件名
    pub filename: String,
    // 文件哈希值
    pub hash: String,
    // 文件类型
    pub media_type: MediaTypeEnum,
    // 是否删除
    pub deleted: bool,
}

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