pub type Value = f64;

#[derive(Debug, Default)]
pub struct ValueArray {
    values: Vec<Value>,
}

impl ValueArray {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn write(&mut self, value: Value) {
        self.values.push(value);
    }

    #[inline]
    pub fn values(&self) -> &[Value] {
        &self.values
    }
}
