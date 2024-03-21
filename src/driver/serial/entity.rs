pub enum SerialThreadCommand {
    WriteData(Vec<u8>),
    Stop
}


#[derive(Debug, Clone)]
pub struct SerialCommandBo {
    pub command: u8,
    pub data: Vec<u8>
}