/// device that can be mounted to serial bus
pub trait SerialMountable {
    /// notify the device, then the device will call upward channel
    fn notify(&self, command: u8, params: Vec<u8>) -> Result<(), DriverError>;
}