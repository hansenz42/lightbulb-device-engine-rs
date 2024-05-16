//! 设备管理模块
//! - 维护设备列表，下载，更新设备列表
//! - 设备操作驱动
//! - 提供设备操作的接口
//! - 定期检查设备状态

pub mod device_manager;
pub mod device_dao;
pub mod device_factory;
pub mod device_info_maker_helper;
mod factory;
mod traits;
pub mod entity;
pub mod thread;