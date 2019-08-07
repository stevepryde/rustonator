use serde_json;

pub trait GameObject {
    fn to_json(&self) -> serde_json::Value;
    fn from_json(&mut self, data: &serde_json::Value);
}

pub struct SuperValue<'a> {
    value: &'a serde_json::Value,
}

impl<'a> SuperValue<'a> {
    pub fn new(value: &'a serde_json::Value) -> Self {
        SuperValue { value }
    }

    pub fn get_value(&self, key: &str) -> &serde_json::Value {
        self.value.get(key).unwrap_or(&serde_json::Value::Null)
    }

    pub fn get_vec(&self, key: &str) -> Vec<serde_json::Value> {
        self.value
            .get(key)
            .map_or(Vec::new(), |x| x.as_array().map(|y| y.clone()).unwrap_or(Vec::new()))
    }

    pub fn get_string(&self, key: &str) -> String {
        self.get_string_or(key, String::new())
    }

    pub fn get_string_or(&self, key: &str, default: String) -> String {
        match self.value.get(key) {
            Some(x) => match x.as_str() {
                Some(y) => y.to_owned(),
                None => default,
            },
            None => default,
        }
    }

    pub fn get_f32(&self, key: &str) -> f32 {
        self.get_f32_or(key, 0.0)
    }

    pub fn get_f32_or(&self, key: &str, default: f32) -> f32 {
        self.value
            .get(key)
            .map_or(default, |x| x.as_f64().map_or(default, |y| y as f32))
    }

    pub fn get_i32(&self, key: &str) -> i32 {
        self.get_i32_or(key, 0)
    }

    pub fn get_i32_or(&self, key: &str, default: i32) -> i32 {
        self.value
            .get(key)
            .map_or(default, |x| x.as_i64().map_or(default, |y| y as i32))
    }

    pub fn get_u32(&self, key: &str) -> u32 {
        self.get_u32_or(key, 0)
    }

    pub fn get_u32_or(&self, key: &str, default: u32) -> u32 {
        self.value
            .get(key)
            .map_or(default, |x| x.as_u64().map_or(default, |y| y as u32))
    }

    pub fn get_bool(&self, key: &str) -> bool {
        self.get_bool_or(key, false)
    }

    pub fn get_bool_or(&self, key: &str, default: bool) -> bool {
        self.value
            .get(key)
            .map_or(default, |x| x.as_bool().unwrap_or(default))
    }
}
