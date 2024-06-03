use super::prelude::*;


// dmx thread command enum, for sending command to dmx thread 
#[derive(Debug)]
pub enum DmxThreadCommandEnum {
    // set channel for the thread
    SetChannel(SetChannelBo),

    // stop the thread
    Stop,
}


#[derive(Debug)]
pub struct SetChannelBo {
    pub channels: [DmxValue; DMX_CHANNEL_LEN]
}