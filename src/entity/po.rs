//! 数据库实体类

#[derive(Debug)]
enum MediaTypeEnum {
    Audio = 1,
    Video = 2
}

/// 数据库对象：文件
#[derive(Debug)]
struct FilePo {
    tag: String,
    orig_filename: String,
    filename: String,
    hash: String,
    media_type: MediaTypeEnum,
    deleted: bool,
}

/// 数据库对象：设备
#[derive(Debug)]
struct DevicePo {
    // 设备二级类目
    device_class: String,
    // 设备类型
    device_type: String,
    // 设备名称
    name: String,
    // 设备描述
    description: String,
    // 设备区域（房间）
    room: String,
    // 设备配置（json string）
    config: String,
}