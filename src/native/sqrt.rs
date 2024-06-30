use crate::value::Value;


pub fn sqrt(_arg_count: u8, values: &[Value]) -> Result<Value, &str> {
    match values[0] {
        Value::Number(num)  => Ok(Value::Number(f64::sqrt(num))),
        _                   => Err("not a number")
    }
}