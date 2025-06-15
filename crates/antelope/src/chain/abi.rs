use std::string::ToString;
use std::fmt;
use crate::serializer::{Decoder, Encoder, Packer, PackerError};
use antelope_client_macros::StructPacker;
use serde::{Deserialize, Serialize};
use crate::{chain::name::{
    serialize_name,
    deserialize_name,
    Name
}, define_error};
use phf::phf_set;

pub static BUILTIN_TYPES: phf::Set<&'static str> = phf_set! {
    "bool",

    "uint8",
    "uint16",
    "uint32",
    "uint64",
    "uint128",

    "int8",
    "int16",
    "int32",
    "int64",
    "int128",

    "varuint32",
    "varint32",

    "float32",
    "float64",
    "float128",

    "time_point",
    "time_point_sec",
    "block_timestamp_type",

    "name",

    "bytes",
    "string",

    "checksum160",
    "checksum256",
    "checksum512",

    "public_key",
    "signature",

    "symbol",
    "symbol_code",

    "asset",
    "extended_asset"
};


#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum TypeModifier {
    Optional,
    Extension,
    Array,
}

impl TypeModifier {
    #[inline]
    pub const fn suffix(&self) -> &'static str {
        match self {
            TypeModifier::Optional  => "?",
            TypeModifier::Extension => "$",
            TypeModifier::Array     => "[]",
        }
    }

    #[inline]
    pub const fn as_str(&self) -> &'static str {
        match self {
            TypeModifier::Optional  => "optional",
            TypeModifier::Extension => "extension",
            TypeModifier::Array     => "array",
        }
    }
}

impl fmt::Display for TypeModifier {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}


#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct ABIResolvedType {
    /// The exact token that the caller passed in (may include modifiers /
    /// alias name / etc.)
    pub original_name: String,

    /// The final concrete type name after chasing every alias
    pub resolved_name: String,

    /// All the aliases we walked through ― outer-most first.
    pub alias_chain: Vec<String>,

    /// Kind flags
    pub is_std: bool,
    pub is_struct: Option<AbiStruct>,
    pub is_variant: Option<AbiVariant>,

    /// Modifiers collected *from every hop*, outer-first order
    pub modifiers: Vec<TypeModifier>,
}

impl ABIResolvedType {
    /// Helper retained for existing callers (`var_meta.is_alias`)
    #[inline]
    pub fn is_alias(&self) -> bool {
        !self.alias_chain.is_empty()
    }
}


fn split_type_modifiers(mut name: &str) -> Result<(String, Vec<TypeModifier>), ABIResolveError> {
    use TypeModifier::*;
    let mut mods = Vec::<TypeModifier>::new();

    loop {
        if let Some(stripped) = name.strip_suffix("[]") {
            mods.push(Array);
            name = stripped;
            continue;
        }
        if let Some(stripped) = name.strip_suffix('?') {
            mods.push(Optional);
            name = stripped;
            continue;
        }
        if let Some(stripped) = name.strip_suffix('$') {
            mods.push(Extension);
            name = stripped;
            continue;
        }
        // detect forbidden fixed-size arrays
        if name.ends_with(']') && name.rfind('[').is_some_and(|lb| name[lb+1..name.len()-1].chars().all(char::is_numeric))
        {
            return Err(ABIResolveError::fmt(format_args!(
                "Fixed-size arrays like “{name}” are not supported")));
        }
        break;
    }
    Ok((name.to_string(), mods))               // outer-first order, like py-jitabi
}

define_error!(ABIResolveError);

pub trait ABITypeResolver {
    fn resolve_type(&self, str: &str) -> Result<ABIResolvedType, ABIResolveError>;
}

pub trait AbiTableView {
    fn name_str(&self) -> String;
    fn type_str(&self) -> String;

    fn key_names(&self) -> Vec<String>;

    fn key_types(&self) -> Option<Vec<String>>;

    fn index_type(&self) -> Option<String>;
}

pub trait ABIView {
    fn types(&self) -> &[AbiTypeDef];
    fn structs(&self) -> &[AbiStruct];
    fn variants(&self) -> &[AbiVariant];
    fn tables(&self) -> &[impl AbiTableView]; // generic over ABI and ShipABI table

    fn actions(&self) -> &[AbiAction];

    fn resolve_alias(&self, type_name: &str) -> Option<String> {
        self.types().iter().find_map(|t| {
            if t.new_type_name == type_name {
                Some(t.r#type.clone())
            } else {
                None
            }
        })
    }
}


impl<ABI: ABIView> ABITypeResolver for ABI {
    fn resolve_type(&self, type_name: &str) -> Result<ABIResolvedType, ABIResolveError> {
        let original = type_name.to_string();
        let (mut base, mut modifiers) = split_type_modifiers(&original)?;

        // follow aliases
        let mut visited = std::collections::HashSet::new();
        let mut chain = Vec::<String>::new();

        while let Some(target) = self.resolve_alias(&base) {
            // guard against cycles
            if !visited.insert(base.clone()) {
                return Err(ABIResolveError::fmt(format_args!(
                    "Circular alias detected: {visited:?} -> {base}"
                )));
            }
            chain.push(base);
            let (next_base, extra_mods) = split_type_modifiers(&target)?;
            modifiers.extend(extra_mods);
            base = next_base;
        }

        // classify
        let is_std = BUILTIN_TYPES.contains(base.as_str());
        let is_struct = self.structs().iter().find(|s| s.name == base).cloned();
        let is_variant = self.variants().iter().find(|v| v.name == base).cloned();

        if !(is_std || is_struct.is_some() || is_variant.is_some()) {
            return Err(ABIResolveError::fmt(format_args!(
                "Unknown type “{base}” after alias resolution"
            )));
        }

        Ok(ABIResolvedType {
            original_name: original,
            resolved_name: base,
            alias_chain: chain,
            is_std,
            is_struct,
            is_variant,
            modifiers,
        })
    }
}


/*

   ABI: application binary interface for on-chain contracts

 */

#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize, StructPacker)]
pub struct ABI {
    pub version: String,
    #[serde(default)]
    pub types: Vec<AbiTypeDef>,
    #[serde(default)]
    pub structs: Vec<AbiStruct>,
    #[serde(default)]
    pub actions: Vec<AbiAction>,
    #[serde(default)]
    pub tables: Vec<AbiTable>,
    #[serde(default)]
    pub ricardian_clauses: Vec<AbiClause>,
    #[serde(default)]
    error_messages: Vec<AbiErrorMessage>,
    #[serde(default)]
    abi_extensions: Vec<String>,
    #[serde(default)]
    pub variants: Vec<AbiVariant>,
    #[serde(default)]
    pub action_results: Vec<AbiActionResult>,
}

impl ABI {
    pub fn from_string(str: &str) -> Result<Self, String> {
        let abi = serde_json::from_str::<ABI>(str)
            .map_err(|e| e.to_string())?;

        Ok(abi)
    }
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize, StructPacker)]
pub struct AbiTypeDef {
    pub new_type_name: String,
    pub r#type: String,
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize, StructPacker)]
pub struct AbiField {
    pub name: String,
    pub r#type: String,
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize, StructPacker)]
pub struct AbiStruct {
    pub name: String,
    #[serde(default)]
    pub base: String,
    pub fields: Vec<AbiField>,
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize, StructPacker)]
pub struct AbiVariant {
    pub name: String,
    pub types: Vec<String>,
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize, StructPacker)]
pub struct AbiAction {
    #[serde(
        serialize_with = "serialize_name",
        deserialize_with = "deserialize_name"
    )]
    pub name: Name,
    pub r#type: String,
    pub ricardian_contract: String,
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize, StructPacker)]
pub struct AbiTable {
    #[serde(
        serialize_with = "serialize_name",
        deserialize_with = "deserialize_name"
    )]
    pub name: Name,
    #[serde(default)]
    pub index_type: String,
    #[serde(default)]
    pub key_names: Vec<String>,
    #[serde(default)]
    pub key_types: Vec<String>,
    pub r#type: String,
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize, StructPacker)]
pub struct AbiClause {
    pub id: String,
    pub body: String,
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize, StructPacker)]
pub struct AbiActionResult {
    #[serde(
        serialize_with = "serialize_name",
        deserialize_with = "deserialize_name"
    )]
    pub name: Name,
    pub result_type: String,
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize, StructPacker)]
pub struct AbiErrorMessage {
    pub error_code: u64,
    pub error_msg: String,
}

/*
    Ship ABI:

    State history plugin's first message on connection is the "standard" ABI encoded as a
    JSON string, the structure is exactly the same as a ABI encoded in JSON except it
    uses String instead of Name for some values allowing it to have this table definition

    "tables": [
        {
            "name": "account_metadata",
            "type": "account_metadata",
            "key_names": [
                "name"
            ]
        }
    ]

    In an ABI that table entry would fail cause "name" key is of type Name
 */

#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize, StructPacker)]
pub struct ShipAbiAction {
    pub name: String,
    pub r#type: String,
    pub ricardian_contract: String,
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize, StructPacker)]
pub struct ShipAbiTable {
    pub name: String,
    #[serde(default)]
    pub index_type: String,
    #[serde(default)]
    pub key_names: Vec<String>,
    #[serde(default)]
    pub key_types: Vec<String>,
    pub r#type: String,
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize, StructPacker)]
pub struct ShipAbiActionResult {
    pub name: Name,
    pub result_type: String,
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize, StructPacker)]
pub struct ShipABI {
    pub version: String,
    #[serde(default)]
    pub types: Vec<AbiTypeDef>,
    #[serde(default)]
    pub structs: Vec<AbiStruct>,
    #[serde(default)]
    pub actions: Vec<ShipAbiAction>,
    #[serde(default)]
    pub tables: Vec<ShipAbiTable>,
    #[serde(default)]
    pub ricardian_clauses: Vec<AbiClause>,
    #[serde(default)]
    error_messages: Vec<AbiErrorMessage>,
    #[serde(default)]
    abi_extensions: Vec<String>,
    #[serde(default)]
    pub variants: Vec<AbiVariant>,
    #[serde(default)]
    pub action_results: Vec<ShipAbiActionResult>,
}

impl ShipABI {
    pub fn from_string(str: &str) -> Result<Self, String> {
        let abi = serde_json::from_str::<ShipABI>(str)
            .map_err(|e| e.to_string())?;

        Ok(abi)
    }
}

impl AbiTableView for AbiTable {
    fn name_str(&self) -> String { self.name.to_string() }
    fn type_str(&self) -> String { self.r#type.clone() }

    fn key_names(&self) -> Vec<String> {
        self.key_names.to_vec()
    }

    fn key_types(&self) -> Option<Vec<String>> {
        Some(self.key_types.to_vec())
    }

    fn index_type(&self) -> Option<String> {
        Some(self.index_type.clone())
    }
}

impl AbiTableView for ShipAbiTable {
    fn name_str(&self) -> String { self.name.clone() }
    fn type_str(&self) -> String { self.r#type.clone() }

    fn key_names(&self) -> Vec<String> {
        self.key_names.to_vec()
    }

    fn key_types(&self) -> Option<Vec<String>> {
        None
    }

    fn index_type(&self) -> Option<String> {
        None
    }
}

impl ABIView for ABI {
    fn types(&self) -> &[AbiTypeDef] { &self.types }
    fn structs(&self) -> &[AbiStruct] { &self.structs }
    fn variants(&self) -> &[AbiVariant] { &self.variants }
    fn tables(&self) -> &[impl AbiTableView] { &self.tables }
    fn actions(&self) -> &[AbiAction] { &self.actions }
}

impl ABIView for ShipABI {
    fn types(&self) -> &[AbiTypeDef] { &self.types }
    fn structs(&self) -> &[AbiStruct] { &self.structs }
    fn variants(&self) -> &[AbiVariant] { &self.variants }
    fn tables(&self) -> &[impl AbiTableView] { &self.tables }
    fn actions(&self) -> &[AbiAction] { &[] }
}
