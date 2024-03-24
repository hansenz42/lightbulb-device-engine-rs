#[derive(Debug, Clone)]
pub enum SerialThreadCommand {
    Write(SerialDataBo),
    // for testing purpose, echo the data from serial bus
    Echo(Vec<u8>),
    Stop
}


#[derive(Debug, Clone)]
pub struct SerialDataBo {
    pub command: u8,
    pub data: Vec<u8>
}