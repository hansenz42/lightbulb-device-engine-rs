pub enum SerialThreadCommand {
    Write(SerialDataBo),
    Stop
}


#[derive(Debug, Clone)]
pub struct SerialDataBo {
    pub command: u8,
    pub data: Vec<u8>
}