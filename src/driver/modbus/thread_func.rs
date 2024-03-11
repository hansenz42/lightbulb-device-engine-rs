//! Modbus 内部线程方法集合
//! 启动线程，打开端口
//! 接收需要轮询的侦听接口信息，并将数据上报给 ModBusBus
//! 如有需要写入接口的数据，则在循环中断，并写入数据


use super::modbus_entity::ModbusThreadCommandBo;
use tokio_serial::SerialStream;

pub async fn run_loop(port: SerialStream) {
    loop {
        
    }
}