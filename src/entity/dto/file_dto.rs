#[derive(Debug, Clone)]
pub enum MediaTypeEnum {
    Audio = 1,
    Video = 2
}

/// used for receiving file config from flow server
#[derive(Debug, Clone)]
pub struct FileDto {
    pub tag: String,
    pub filename: String,
    pub hash: String,
    pub media_type: MediaTypeEnum,
    pub deleted: bool,
}