/// 数据库对象：文件
use crate::entity::bo::file_bo::MediaTypeEnum;

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