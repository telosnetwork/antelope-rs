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

    pub fn bool(v: Option<&Value>) -> Result<bool, EncodingError> {
        check_some(v, "bool")?;
        let value = v.unwrap();
        if !value.is_boolean() {
            return Err(EncodingError::new("Value is not a boolean".into()));
        }

        Ok(value.as_bool().unwrap())
    }

    pub fn json_object(v: Option<&Value>) -> Result<JSONObject, EncodingError> {
        check_some(v, "JSON object")?;
        let value = v.unwrap();
        if !value.is_object() {
            return Err(EncodingError::new("Value is not a JSON object".into()));
        }

        Ok(JSONObject::new(value.clone()))
    }
}

#[derive(Debug)]
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

    pub fn get_optional_string(&self, property: &str) -> Result<Option<String>, EncodingError> {
        match self.value.get(property) {
            Some(v) => match v.as_str() {
                Some(s) => Ok(Some(s.to_string())),
                None => Ok(None),
            },
            None => Ok(None),
        }
    }

    pub fn get_optional_u32(&self, property: &str) -> Result<Option<u32>, EncodingError> {
        match self.value.get(property) {
            Some(v) => match v.as_u64() {
                Some(n) => Ok(Some(n as u32)),
                None => Ok(None),
            },
            None => Ok(None),
        }
    }

    pub fn get_str(&self, property: &str) -> Result<&str, EncodingError> {
        ValueTo::str(self.value.get(property))
    }

    pub fn get_string(&self, property: &str) -> Result<String, EncodingError> {
        ValueTo::string(self.value.get(property))
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

    pub fn get_bool(&self, property: &str) -> Result<bool, EncodingError> {
        ValueTo::bool(self.value.get(property))
    }

    pub fn get_json_object(&self, property: &str) -> Result<JSONObject, EncodingError> {
        ValueTo::json_object(self.value.get(property))
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
