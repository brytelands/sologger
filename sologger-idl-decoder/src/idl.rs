use serde::Deserialize;
use serde_json::Value;

use crate::decoder::{event_discriminator, DecodeError};

/// A parsed Anchor IDL. One serde model covers both the legacy (pre-0.30) spec and the
/// 0.30+ spec; the accessors below paper over the differences:
///
/// - legacy keeps `name`/`version` at the top level, 0.30+ nests them under `metadata`
/// - legacy events carry inline `fields`, 0.30+ events carry an explicit `discriminator`
///   and define their fields as a struct of the same name in `types`
#[derive(Deserialize, Clone, Debug, Default)]
pub struct Idl {
    ///Program name (legacy spec top-level field)
    #[serde(default)]
    pub name: Option<String>,
    ///IDL version (legacy spec top-level field)
    #[serde(default)]
    pub version: Option<String>,
    ///Program address (0.30+ spec)
    #[serde(default)]
    pub address: Option<String>,
    ///Metadata block holding name/version/spec (0.30+ spec)
    #[serde(default)]
    pub metadata: Option<IdlMetadata>,
    ///Events the program can emit
    #[serde(default)]
    pub events: Vec<IdlEvent>,
    ///Custom error codes the program can fail with
    #[serde(default)]
    pub errors: Vec<IdlErrorCode>,
    ///Named type definitions referenced by events, accounts and instruction args
    #[serde(default)]
    pub types: Vec<IdlTypeDef>,
}

impl Idl {
    /// Parses an IDL from its JSON text, accepting either spec version.
    pub fn from_json(json: &str) -> Result<Self, DecodeError> {
        serde_json::from_str(json).map_err(|e| DecodeError::InvalidIdl(e.to_string()))
    }

    /// The program name, from whichever spec location holds it.
    pub fn program_name(&self) -> &str {
        self.name
            .as_deref()
            .or_else(|| self.metadata.as_ref().and_then(|m| m.name.as_deref()))
            .unwrap_or("")
    }

    /// Looks up a named type definition.
    pub fn find_type(&self, name: &str) -> Option<&IdlTypeDef> {
        self.types.iter().find(|t| t.name == name)
    }

    /// Looks up an error by its numeric code (e.g. 6001 for Anchor error 0x1771).
    pub fn lookup_error(&self, code: u32) -> Option<&IdlErrorCode> {
        self.errors.iter().find(|e| e.code == code)
    }
}

/// The `metadata` block of a 0.30+ spec IDL.
#[derive(Deserialize, Clone, Debug, Default)]
pub struct IdlMetadata {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub spec: Option<String>,
}

/// An event definition. Legacy IDLs inline the fields; 0.30+ IDLs carry an explicit
/// discriminator and define the fields as a struct in `types`.
#[derive(Deserialize, Clone, Debug)]
pub struct IdlEvent {
    pub name: String,
    #[serde(default)]
    pub discriminator: Option<Vec<u8>>,
    #[serde(default)]
    pub fields: Option<Vec<IdlField>>,
}

impl IdlEvent {
    /// The 8-byte discriminator that prefixes this event's borsh payload: the explicit
    /// one when the IDL provides it (0.30+), otherwise sha256("event:<Name>")[..8]
    /// exactly as anchor-lang derives it for legacy programs.
    pub fn discriminator_bytes(&self) -> [u8; 8] {
        match &self.discriminator {
            Some(explicit) if explicit.len() == 8 => {
                let mut out = [0u8; 8];
                out.copy_from_slice(explicit);
                out
            }
            _ => event_discriminator(&self.name),
        }
    }
}

/// An entry in the IDL's `errors` array. Identical in both specs.
#[derive(Deserialize, Clone, Debug)]
pub struct IdlErrorCode {
    pub code: u32,
    pub name: String,
    #[serde(default)]
    pub msg: Option<String>,
}

/// A named type definition from the IDL's `types` array.
#[derive(Deserialize, Clone, Debug)]
pub struct IdlTypeDef {
    pub name: String,
    #[serde(rename = "type")]
    pub ty: IdlTypeDefTy,
}

/// The body of a type definition: struct, enum, or (0.30+) type alias.
#[derive(Deserialize, Clone, Debug)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum IdlTypeDefTy {
    Struct {
        #[serde(default)]
        fields: Option<IdlDefinedFields>,
    },
    Enum {
        variants: Vec<IdlEnumVariant>,
    },
    Alias {
        value: IdlType,
    },
}

/// One variant of an enum type definition.
#[derive(Deserialize, Clone, Debug)]
pub struct IdlEnumVariant {
    pub name: String,
    #[serde(default)]
    pub fields: Option<IdlDefinedFields>,
}

/// Struct or enum-variant fields: named (`{"name": .., "type": ..}` objects) or a
/// tuple (bare types).
#[derive(Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum IdlDefinedFields {
    Named(Vec<IdlField>),
    Tuple(Vec<IdlType>),
}

/// A single named field.
#[derive(Deserialize, Clone, Debug)]
pub struct IdlField {
    pub name: String,
    #[serde(rename = "type")]
    pub ty: IdlType,
}

/// A type reference as it appears in field positions. Primitives are strings; compound
/// types are single-key objects. Unknown or not-yet-supported shapes parse into
/// `Unsupported` so one exotic field only fails decoding, never IDL parsing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IdlType {
    Bool,
    U8,
    I8,
    U16,
    I16,
    U32,
    I32,
    U64,
    I64,
    U128,
    I128,
    F32,
    F64,
    Bytes,
    String,
    ///"publicKey" in the legacy spec, "pubkey" in 0.30+
    Pubkey,
    Vec(Box<IdlType>),
    Option(Box<IdlType>),
    Array(Box<IdlType>, usize),
    ///Reference to a named entry in `types`. Legacy: {"defined": "Name"}; 0.30+: {"defined": {"name": "Name"}}
    Defined(std::string::String),
    Unsupported(std::string::String),
}

impl<'de> Deserialize<'de> for IdlType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        Ok(IdlType::from_value(&value))
    }
}

impl IdlType {
    fn from_value(value: &Value) -> IdlType {
        match value {
            Value::String(s) => match s.as_str() {
                "bool" => IdlType::Bool,
                "u8" => IdlType::U8,
                "i8" => IdlType::I8,
                "u16" => IdlType::U16,
                "i16" => IdlType::I16,
                "u32" => IdlType::U32,
                "i32" => IdlType::I32,
                "u64" => IdlType::U64,
                "i64" => IdlType::I64,
                "u128" => IdlType::U128,
                "i128" => IdlType::I128,
                "f32" => IdlType::F32,
                "f64" => IdlType::F64,
                "bytes" => IdlType::Bytes,
                "string" => IdlType::String,
                "pubkey" | "publicKey" => IdlType::Pubkey,
                other => IdlType::Unsupported(other.to_string()),
            },
            Value::Object(map) => {
                if let Some(inner) = map.get("vec") {
                    IdlType::Vec(Box::new(IdlType::from_value(inner)))
                } else if let Some(inner) = map.get("option") {
                    IdlType::Option(Box::new(IdlType::from_value(inner)))
                } else if let Some(arr) = map.get("array").and_then(|a| a.as_array()) {
                    match (arr.first(), arr.get(1).and_then(|l| l.as_u64())) {
                        (Some(elem), Some(len)) => {
                            IdlType::Array(Box::new(IdlType::from_value(elem)), len as usize)
                        }
                        _ => IdlType::Unsupported(value.to_string()),
                    }
                } else if let Some(defined) = map.get("defined") {
                    if let Some(name) = defined.as_str() {
                        IdlType::Defined(name.to_string())
                    } else if let Some(name) = defined.get("name").and_then(|n| n.as_str()) {
                        let has_generics = defined
                            .get("generics")
                            .and_then(|g| g.as_array())
                            .is_some_and(|g| !g.is_empty());
                        if has_generics {
                            IdlType::Unsupported(value.to_string())
                        } else {
                            IdlType::Defined(name.to_string())
                        }
                    } else {
                        IdlType::Unsupported(value.to_string())
                    }
                } else {
                    IdlType::Unsupported(value.to_string())
                }
            }
            other => IdlType::Unsupported(other.to_string()),
        }
    }
}
