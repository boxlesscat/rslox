use crate::value::Value;

use std::time::SystemTime;
use std::time::UNIX_EPOCH;

pub fn clock(_arg_count: u8, _values: &[Value]) -> Result<Value, &str> {
    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| "SystemTime before UNIX EPOCH!")?
        .as_millis() as f64;
    Ok(Value::Number(time))
}