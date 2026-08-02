use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};

use crate::idl::{Idl, IdlDefinedFields, IdlEvent, IdlField, IdlType, IdlTypeDefTy};

/// Decoding stops recursing into nested `defined`/`option`/`vec` types beyond this depth.
/// Real event payloads nest a handful of levels; the cap only guards against pathological
/// IDLs, and matters on WASM where the stack is small.
const MAX_DEPTH: usize = 64;

/// An Anchor event decoded out of a 'Program data:' log line.
#[derive(Debug, Clone, PartialEq)]
pub struct DecodedEvent {
    ///The event name as declared in the IDL
    pub name: String,
    ///The decoded fields. u128/i128 values are rendered as decimal strings, pubkeys as
    ///base58 strings, and `bytes` fields as base64 strings
    pub data: Value,
}

impl DecodedEvent {
    /// Renders the event as a compact JSON string of the form {"name":...,"data":{...}},
    /// the format stored in `LogContext::decoded_events`.
    pub fn to_json(&self) -> String {
        serde_json::json!({ "name": self.name, "data": self.data }).to_string()
    }
}

/// Why a decode attempt failed.
#[derive(Debug)]
pub enum DecodeError {
    ///The IDL JSON itself could not be parsed
    InvalidIdl(String),
    ///The event payload was malformed (bad base64, truncated fields, invalid tags)
    InvalidData(String),
    ///A `defined` type reference has no entry in the IDL's `types` array
    UnknownType(String),
    ///The field uses a type this decoder does not support (e.g. generics, u256)
    UnsupportedType(String),
}

impl std::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            DecodeError::InvalidIdl(msg) => write!(f, "invalid IDL: {}", msg),
            DecodeError::InvalidData(msg) => write!(f, "invalid event data: {}", msg),
            DecodeError::UnknownType(name) => write!(f, "unknown type in IDL: {}", name),
            DecodeError::UnsupportedType(name) => write!(f, "unsupported IDL type: {}", name),
        }
    }
}

impl std::error::Error for DecodeError {}

/// The discriminator anchor-lang derives for an event name: sha256("event:<Name>")[..8].
/// 0.30+ IDLs list the same bytes explicitly; legacy IDLs rely on this computation.
pub fn event_discriminator(name: &str) -> [u8; 8] {
    let mut hasher = Sha256::new();
    hasher.update(b"event:");
    hasher.update(name.as_bytes());
    let digest = hasher.finalize();
    let mut out = [0u8; 8];
    out.copy_from_slice(&digest[..8]);
    out
}

/// Attempts to decode one base64 'Program data:' payload against the IDL's events.
///
/// Returns Ok(None) when the payload is well-formed but matches no event discriminator
/// (common: programs also emit non-event data), and Err when a matched event's payload
/// cannot be decoded.
pub fn decode_event(idl: &Idl, data_b64: &str) -> Result<Option<DecodedEvent>, DecodeError> {
    let bytes = BASE64
        .decode(data_b64.trim())
        .map_err(|e| DecodeError::InvalidData(format!("base64: {}", e)))?;
    if bytes.len() < 8 {
        return Ok(None);
    }

    for event in &idl.events {
        if bytes[..8] == event.discriminator_bytes() {
            let mut reader = Reader::new(&bytes[8..]);
            let data = decode_event_fields(idl, event, &mut reader)?;
            return Ok(Some(DecodedEvent {
                name: event.name.clone(),
                data,
            }));
        }
    }
    Ok(None)
}

/// Decodes every payload in `data_logs`, silently skipping entries that match no event
/// or fail to decode. The lossy convenience wrapper used for log enrichment.
pub fn decode_events(idl: &Idl, data_logs: &[String]) -> Vec<DecodedEvent> {
    data_logs
        .iter()
        .filter_map(|data| match decode_event(idl, data) {
            Ok(found) => found,
            Err(err) => {
                log::debug!("skipping undecodable data log: {}", err);
                None
            }
        })
        .collect()
}

fn decode_event_fields(
    idl: &Idl,
    event: &IdlEvent,
    reader: &mut Reader,
) -> Result<Value, DecodeError> {
    match &event.fields {
        // Legacy spec: fields inline on the event
        Some(fields) => decode_named_fields(idl, fields, reader, 0),
        // 0.30+ spec: fields live in a type definition of the same name
        None => decode_defined(idl, &event.name, reader, 0),
    }
}

fn decode_defined(
    idl: &Idl,
    name: &str,
    reader: &mut Reader,
    depth: usize,
) -> Result<Value, DecodeError> {
    if depth > MAX_DEPTH {
        return Err(DecodeError::InvalidData(
            "max nesting depth exceeded".into(),
        ));
    }
    let type_def = idl
        .find_type(name)
        .ok_or_else(|| DecodeError::UnknownType(name.to_string()))?;
    match &type_def.ty {
        IdlTypeDefTy::Struct { fields } => {
            decode_defined_fields(idl, fields.as_ref(), reader, depth)
        }
        IdlTypeDefTy::Enum { variants } => {
            let index = reader.read_u8()? as usize;
            let variant = variants.get(index).ok_or_else(|| {
                DecodeError::InvalidData(format!("enum {} has no variant {}", name, index))
            })?;
            match &variant.fields {
                None => Ok(Value::String(variant.name.clone())),
                Some(fields) => {
                    let inner = decode_defined_fields(idl, Some(fields), reader, depth)?;
                    let mut map = Map::new();
                    map.insert(variant.name.clone(), inner);
                    Ok(Value::Object(map))
                }
            }
        }
        IdlTypeDefTy::Alias { value } => decode_type(idl, value, reader, depth + 1),
    }
}

fn decode_defined_fields(
    idl: &Idl,
    fields: Option<&IdlDefinedFields>,
    reader: &mut Reader,
    depth: usize,
) -> Result<Value, DecodeError> {
    match fields {
        None => Ok(Value::Object(Map::new())),
        Some(IdlDefinedFields::Named(fields)) => decode_named_fields(idl, fields, reader, depth),
        Some(IdlDefinedFields::Tuple(types)) => {
            let mut items = Vec::with_capacity(types.len());
            for ty in types {
                items.push(decode_type(idl, ty, reader, depth + 1)?);
            }
            Ok(Value::Array(items))
        }
    }
}

fn decode_named_fields(
    idl: &Idl,
    fields: &[IdlField],
    reader: &mut Reader,
    depth: usize,
) -> Result<Value, DecodeError> {
    let mut map = Map::new();
    for field in fields {
        let value = decode_type(idl, &field.ty, reader, depth + 1)?;
        map.insert(field.name.clone(), value);
    }
    Ok(Value::Object(map))
}

fn decode_type(
    idl: &Idl,
    ty: &IdlType,
    reader: &mut Reader,
    depth: usize,
) -> Result<Value, DecodeError> {
    if depth > MAX_DEPTH {
        return Err(DecodeError::InvalidData(
            "max nesting depth exceeded".into(),
        ));
    }
    match ty {
        IdlType::Bool => match reader.read_u8()? {
            0 => Ok(Value::Bool(false)),
            1 => Ok(Value::Bool(true)),
            other => Err(DecodeError::InvalidData(format!(
                "invalid bool tag {}",
                other
            ))),
        },
        IdlType::U8 => Ok(Value::from(reader.read_u8()?)),
        IdlType::I8 => Ok(Value::from(reader.read_i8()?)),
        IdlType::U16 => Ok(Value::from(reader.read_u16()?)),
        IdlType::I16 => Ok(Value::from(reader.read_i16()?)),
        IdlType::U32 => Ok(Value::from(reader.read_u32()?)),
        IdlType::I32 => Ok(Value::from(reader.read_i32()?)),
        IdlType::U64 => Ok(Value::from(reader.read_u64()?)),
        IdlType::I64 => Ok(Value::from(reader.read_i64()?)),
        // 128-bit integers exceed JSON number precision; render as decimal strings
        IdlType::U128 => Ok(Value::String(reader.read_u128()?.to_string())),
        IdlType::I128 => Ok(Value::String(reader.read_i128()?.to_string())),
        IdlType::F32 => Ok(Value::from(reader.read_f32()? as f64)),
        IdlType::F64 => Ok(Value::from(reader.read_f64()?)),
        IdlType::Bytes => {
            let len = reader.read_len()?;
            let bytes = reader.take(len)?;
            Ok(Value::String(BASE64.encode(bytes)))
        }
        IdlType::String => {
            let len = reader.read_len()?;
            let bytes = reader.take(len)?;
            Ok(Value::String(String::from_utf8_lossy(bytes).into_owned()))
        }
        IdlType::Pubkey => {
            let bytes = reader.take(32)?;
            Ok(Value::String(bs58::encode(bytes).into_string()))
        }
        IdlType::Vec(elem) => {
            let len = reader.read_len()?;
            let mut items = Vec::new();
            for _ in 0..len {
                items.push(decode_type(idl, elem, reader, depth + 1)?);
            }
            Ok(Value::Array(items))
        }
        IdlType::Option(inner) => match reader.read_u8()? {
            0 => Ok(Value::Null),
            1 => decode_type(idl, inner, reader, depth + 1),
            other => Err(DecodeError::InvalidData(format!(
                "invalid option tag {}",
                other
            ))),
        },
        IdlType::Array(elem, len) => {
            let mut items = Vec::with_capacity((*len).min(4096));
            for _ in 0..*len {
                items.push(decode_type(idl, elem, reader, depth + 1)?);
            }
            Ok(Value::Array(items))
        }
        IdlType::Defined(name) => decode_defined(idl, name, reader, depth + 1),
        IdlType::Unsupported(desc) => Err(DecodeError::UnsupportedType(desc.clone())),
    }
}

/// Little-endian cursor over a borsh payload.
struct Reader<'a> {
    data: &'a [u8],
    pos: usize,
}

macro_rules! read_le {
    ($name:ident, $ty:ty) => {
        fn $name(&mut self) -> Result<$ty, DecodeError> {
            let bytes = self.take(std::mem::size_of::<$ty>())?;
            Ok(<$ty>::from_le_bytes(bytes.try_into().unwrap()))
        }
    };
}

impl<'a> Reader<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    fn remaining(&self) -> usize {
        self.data.len() - self.pos
    }

    fn take(&mut self, len: usize) -> Result<&'a [u8], DecodeError> {
        if len > self.remaining() {
            return Err(DecodeError::InvalidData(format!(
                "payload truncated: wanted {} bytes at offset {}, {} remain",
                len,
                self.pos,
                self.remaining()
            )));
        }
        let slice = &self.data[self.pos..self.pos + len];
        self.pos += len;
        Ok(slice)
    }

    /// Reads a borsh u32 length prefix, rejecting lengths that exceed the remaining
    /// payload so corrupt data cannot trigger runaway loops.
    fn read_len(&mut self) -> Result<usize, DecodeError> {
        let len = self.read_u32()? as usize;
        if len > self.remaining() {
            return Err(DecodeError::InvalidData(format!(
                "length prefix {} exceeds remaining payload {}",
                len,
                self.remaining()
            )));
        }
        Ok(len)
    }

    read_le!(read_u16, u16);
    read_le!(read_u32, u32);
    read_le!(read_u64, u64);
    read_le!(read_u128, u128);
    read_le!(read_i8, i8);
    read_le!(read_i16, i16);
    read_le!(read_i32, i32);
    read_le!(read_i64, i64);
    read_le!(read_i128, i128);
    read_le!(read_f32, f32);
    read_le!(read_f64, f64);

    fn read_u8(&mut self) -> Result<u8, DecodeError> {
        Ok(self.take(1)?[0])
    }
}
