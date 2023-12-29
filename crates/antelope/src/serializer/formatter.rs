use serde_json::Value;
use crate::util::hex_to_bytes;

pub struct ValueTo {
}

impl ValueTo {

    pub fn str(v: Option<&Value>) -> Result<&str, String> {
        check_some(v, "str")?;
        let value = v.unwrap();
        if !value.is_string() {
            return Err(String::from("Value is not String"));
        }

        Ok(value.as_str().unwrap())
    }

    pub fn string(v: Option<&Value>) -> Result<String, String> {
        check_some(v, "String")?;
        let value = v.unwrap();
        if !value.is_string() {
            return Err(String::from("Value is not String"));
        }

        Ok(value.as_str().unwrap().to_string())
    }

    pub fn hex_bytes(v: Option<&Value>) -> Result<Vec<u8>, String> {
        let value = Self::string(v)?;
        return Ok(hex_to_bytes(value.as_str()));
    }

    pub fn u32(v: Option<&Value>) -> Result<u32, String> {
        check_some(v, "u32")?;
        let value = v.unwrap();
        if !value.is_number() {
            return Err(String::from("Value is not a number"));
        }

        Ok(value.as_number().unwrap().as_u64().unwrap() as u32)
    }

    pub fn u64(v: Option<&Value>) -> Result<u64, String> {
        check_some(v, "u64")?;
        let value = v.unwrap();
        if !value.is_number() {
            return Err(String::from("Value is not a number"));
        }

        Ok(value.as_number().unwrap().as_u64().unwrap())
    }

}

pub struct JSONObject {
    value: Value
}

impl JSONObject {

    pub fn new(value: Value) -> Self {
        JSONObject {
            value
        }
    }

    pub fn get_str(&self, property: &str) -> Result<&str, String> {
        ValueTo::str(self.value.get(property))
    }

    pub fn get_string(&self, property: &str) -> Result<String, String> {
        ValueTo::string(self.value.get(property))
    }

    pub fn get_hex_bytes(&self, property: &str) -> Result<Vec<u8>, String> {
        ValueTo::hex_bytes(self.value.get(property))
    }

    pub fn get_u32(&self, property: &str) -> Result<u32, String> {
        ValueTo::u32(self.value.get(property))
    }

    pub fn get_u64(&self, property: &str) -> Result<u64, String> {
        ValueTo::u64(self.value.get(property))
    }


}

pub fn check_some(o: Option<&Value>, type_name: &str) -> Result<String, String> {
    if o.is_none() {
        return Err(String::from("Value is None, cannot convert to ") + type_name);
    }

    Ok(String::from(""))
}