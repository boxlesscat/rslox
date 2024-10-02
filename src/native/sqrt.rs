use crate::value::Value;


pub fn sqrt(_arg_count: u8, values: &[Value]) -> Result<Value, String> {
    match &values[0] {
        Value::Number(num)  => Ok(Value::Number(f64::sqrt(num.clone()))),
        value               => Err(format!("{value} is not a number")),
    }
}