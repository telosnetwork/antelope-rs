use crate::api::v1::structs::EncodingError;
use crate::util::hex_to_bytes;
use serde_json::Value;

pub struct ValueTo {}

impl ValueTo {
    pub fn str(v: Option<&Value>) -> Result<&str, EncodingError> {
        check_some(v, "str")?;
        let value = v.unwrap();
        if !value.is_string() {
            return Err(EncodingError::new("Value is not String".into()));
        }

        Ok(value.as_str().unwrap())
    }

    pub fn string(v: Option<&Value>) -> Result<String, EncodingError> {
        check_some(v, "String")?;
        let value = v.unwrap();
        if !value.is_string() {
            return Err(EncodingError::new("Value is not String".into()));
        }

        Ok(value.as_str().unwrap().to_string())
    }

    pub fn bool(v: Option<&Value>) -> Result<bool, EncodingError> {
        check_some(v, "bool")?;
        let value = v.unwrap();
        if !value.is_boolean() {
            return Err(EncodingError::new("Value is not bool".into()));
        }

        Ok(value.as_bool().unwrap())
    }

    pub fn vec(v: Option<&Value>) -> Result<&Vec<Value>, EncodingError> {
        check_some(v, "Vec")?;
        let value = v.unwrap();
        if !value.is_array() {
            return Err(EncodingError::new("Value is not Vec".into()));
        }

        Ok(value.as_array().unwrap())
    }

    pub fn hex_bytes(v: Option<&Value>) -> Result<Vec<u8>, EncodingError> {
        let value = Self::string(v)?;
        return Ok(hex_to_bytes(value.as_str()));
    }

    pub fn u32(v: Option<&Value>) -> Result<u32, EncodingError> {
        check_some(v, "u32")?;
        let value = v.unwrap();
        if !value.is_number() {
            return Err(EncodingError::new("Value is not a number".into()));
        }

        Ok(value.as_number().unwrap().as_u64().unwrap() as u32)
    }

    pub fn u64(v: Option<&Value>) -> Result<u64, EncodingError> {
        check_some(v, "u64")?;
        let value = v.unwrap();
        if !value.is_number() {
            return Err(EncodingError::new("Value is not a number".into()));
        }

        Ok(value.as_number().unwrap().as_u64().unwrap())
    }
}

pub struct JSONObject {
    value: Value,
}

impl JSONObject {
    pub fn new(value: Value) -> Self {
        JSONObject { value }
    }

    pub fn has(&self, property: &str) -> bool {
        self.value.get(property).is_some()
    }

    pub fn get_value(&self, property: &str) -> Result<Value, EncodingError> {
        let value = self.value.get(property);
        if value.is_none() {
            return Err(EncodingError::new(format!(
                "Unable to get property {}",
                property
            )));
        }

        Ok(value.unwrap().clone())
    }

    pub fn get_str(&self, property: &str) -> Result<&str, EncodingError> {
        ValueTo::str(self.value.get(property))
    }

    pub fn get_string(&self, property: &str) -> Result<String, EncodingError> {
        ValueTo::string(self.value.get(property))
    }

    pub fn get_bool(&self, property: &str) -> Result<bool, EncodingError> {
        ValueTo::bool(self.value.get(property))
    }

    pub fn get_vec(&self, property: &str) -> Result<&Vec<Value>, EncodingError> {
        ValueTo::vec(self.value.get(property))
    }

    pub fn get_hex_bytes(&self, property: &str) -> Result<Vec<u8>, EncodingError> {
        ValueTo::hex_bytes(self.value.get(property))
    }

    pub fn get_u32(&self, property: &str) -> Result<u32, EncodingError> {
        ValueTo::u32(self.value.get(property))
    }

    pub fn get_u64(&self, property: &str) -> Result<u64, EncodingError> {
        ValueTo::u64(self.value.get(property))
    }
}

pub fn check_some(o: Option<&Value>, type_name: &str) -> Result<String, EncodingError> {
    if o.is_none() {
        return Err(EncodingError::new(format!(
            "Value is None, cannot convert to {}",
            type_name
        )));
    }

    Ok(String::from(""))
}
