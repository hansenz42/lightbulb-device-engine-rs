/// 数据库对象：文件
use crate::entity::dto::file_dto::MediaTypeEnum;

#[derive(Debug, Clone)]
pub struct FilePo {
    pub tag: String,
    pub filename: String,
    pub hash: String,
    pub media_type: MediaTypeEnum,
    pub deleted: bool,
}