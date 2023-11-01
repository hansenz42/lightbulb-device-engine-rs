/// 需要挂载在其他设备上的设备
use super::master::Master;

pub trait Slave {
   fn mount_to_master(&mut self, master_device: Master) {}
}