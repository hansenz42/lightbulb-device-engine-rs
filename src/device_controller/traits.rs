use serde_json::Value;

// the factory that can make bus device
pub trait MakeBus {
    fn make(json_value: Value) -> Box<dyn Bus>;
}

pub trait MakeBusMountable {
    fn make(json_value: Value) -> Box<dyn BusMountable>;
}