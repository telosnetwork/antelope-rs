use serde::{Deserialize, Serialize};

use crate::{
    chain::name::{deserialize_name, Name},
    // serializer::{Decoder, Encoder, Packer},
};

#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct ABI {
    pub version: String,
    pub types: Option<Vec<AbiTypeDef>>,
    pub structs: Vec<AbiStruct>,
    variants: Option<Vec<AbiVarient>>,
    pub actions: Vec<AbiAction>,
    pub tables: Vec<AbiTable>,
    ricardian_clauses: Option<Vec<AbiClause>>,
    action_results: Option<Vec<AbiActionResult>>,
    // error_messages: [],
    // abi_extensions: [],
    // kv_tables: {}
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct AbiTypeDef {
    new_type_name: String,
    r#type: String,
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct AbiField {
    pub name: String,
    pub r#type: String,
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct AbiStruct {
    pub name: String,
    base: String,
    pub fields: Vec<AbiField>,
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct AbiVarient {
    name: String,
    types: Vec<String>,
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct AbiAction {
    #[serde(deserialize_with = "deserialize_name")]
    pub name: Name,
    pub r#type: String,
    ricardian_contract: String,
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct AbiTable {
    #[serde(deserialize_with = "deserialize_name")]
    pub name: Name,
    index_type: String,
    key_names: Option<Vec<String>>,
    key_types: Option<Vec<String>>,
    r#type: String,
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct AbiClause {
    id: String,
    body: String,
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct AbiActionResult {
    #[serde(deserialize_with = "deserialize_name")]
    name: Name,
    result_type: String,
}
