//! Bounded raw JSON content validation. A parsed value carries no execution authority.
use serde::de::{self, DeserializeSeed, MapAccess, SeqAccess, Visitor};
use serde_json::{Map, Number, Value};
use std::fmt;

/// Inclusive budgets. Keys count toward decoded bytes, not value nodes.
#[derive(Clone, Copy, Debug)]
pub struct Limits {
    pub frame_bytes: usize,
    pub depth: usize,
    pub nodes: usize,
    pub string_bytes: usize,
    pub total_string_bytes: usize,
    pub object_entries: usize,
    pub array_entries: usize,
}
impl Limits {
    /// Compiled ceilings, not a recommended profile for any particular consumer.
    pub const CEILINGS: Self = Self {
        frame_bytes: 4 * 1024 * 1024,
        depth: 32,
        nodes: 65_536,
        string_bytes: 1024 * 1024,
        total_string_bytes: 4 * 1024 * 1024,
        object_entries: 4096,
        array_entries: 4096,
    };
    fn valid(self) -> bool {
        let c = Self::CEILINGS;
        [
            (self.frame_bytes, c.frame_bytes),
            (self.depth, c.depth),
            (self.nodes, c.nodes),
            (self.string_bytes, c.string_bytes),
            (self.total_string_bytes, c.total_string_bytes),
            (self.object_entries, c.object_entries),
            (self.array_entries, c.array_entries),
        ]
        .into_iter()
        .all(|(v, max)| v > 0 && v <= max)
    }
}

/// Finite diagnostics deliberately exclude parser messages and input content.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    InvalidLimits,
    FrameBytes,
    InvalidJson,
    Depth,
    Nodes,
    StringBytes,
    TotalStringBytes,
    ObjectEntries,
    ArrayEntries,
    DuplicateKey,
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "strict JSON: {self:?}")
    }
}
impl std::error::Error for Error {}

/// Reject duplicates on original bytes before any Value conversion loses them.
/// Typed schemas and genuine owned-result bindings must still be checked later.
pub fn decode(bytes: &[u8], limits: Limits) -> Result<Value, Error> {
    if !limits.valid() {
        return Err(Error::InvalidLimits);
    }
    if bytes.len() > limits.frame_bytes {
        return Err(Error::FrameBytes);
    }
    let mut budget = Budget {
        limits,
        nodes: 0,
        strings: 0,
        error: None,
    };
    let mut decoder = serde_json::Deserializer::from_slice(bytes);
    let parsed = Seed {
        budget: &mut budget,
        depth: 0,
        entry_error: None,
    }
    .deserialize(&mut decoder);
    match parsed {
        Ok(value) => decoder
            .end()
            .map(|()| value)
            .map_err(|_| Error::InvalidJson),
        Err(_) => Err(budget.error.unwrap_or(Error::InvalidJson)),
    }
}

struct Budget {
    limits: Limits,
    nodes: usize,
    strings: usize,
    error: Option<Error>,
}
impl Budget {
    fn refuse<E: de::Error>(&mut self, kind: Error) -> E {
        self.error.get_or_insert(kind);
        E::custom("strict JSON rejected")
    }
    fn node<E: de::Error>(&mut self) -> Result<(), E> {
        self.nodes = self
            .nodes
            .checked_add(1)
            .filter(|n| *n <= self.limits.nodes)
            .ok_or_else(|| self.refuse(Error::Nodes))?;
        Ok(())
    }
    fn string<E: de::Error>(&mut self, text: &str) -> Result<(), E> {
        if text.len() > self.limits.string_bytes {
            return Err(self.refuse(Error::StringBytes));
        }
        self.strings = self
            .strings
            .checked_add(text.len())
            .filter(|n| *n <= self.limits.total_string_bytes)
            .ok_or_else(|| self.refuse(Error::TotalStringBytes))?;
        Ok(())
    }
}
struct Seed<'a> {
    budget: &'a mut Budget,
    depth: usize,
    entry_error: Option<Error>,
}
impl<'de> DeserializeSeed<'de> for Seed<'_> {
    type Value = Value;
    fn deserialize<D: de::Deserializer<'de>>(self, decoder: D) -> Result<Value, D::Error> {
        if let Some(error) = self.entry_error {
            return Err(self.budget.refuse(error));
        }
        self.budget.node()?;
        decoder.deserialize_any(self)
    }
}
impl<'de> Visitor<'de> for Seed<'_> {
    type Value = Value;
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("bounded JSON value")
    }
    fn visit_bool<E: de::Error>(self, v: bool) -> Result<Value, E> {
        Ok(Value::Bool(v))
    }
    fn visit_unit<E: de::Error>(self) -> Result<Value, E> {
        Ok(Value::Null)
    }
    fn visit_i64<E: de::Error>(self, v: i64) -> Result<Value, E> {
        Ok(Value::Number(v.into()))
    }
    fn visit_u64<E: de::Error>(self, v: u64) -> Result<Value, E> {
        Ok(Value::Number(v.into()))
    }
    fn visit_f64<E: de::Error>(self, v: f64) -> Result<Value, E> {
        Number::from_f64(v)
            .map(Value::Number)
            .ok_or_else(|| self.budget.refuse(Error::InvalidJson))
    }
    fn visit_str<E: de::Error>(self, v: &str) -> Result<Value, E> {
        self.budget.string(v)?;
        Ok(Value::String(v.to_owned()))
    }
    fn visit_string<E: de::Error>(self, v: String) -> Result<Value, E> {
        self.budget.string(&v)?;
        Ok(Value::String(v))
    }
    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Value, A::Error> {
        let depth = self.depth.checked_add(1);
        let Some(depth) = depth.filter(|n| *n <= self.budget.limits.depth) else {
            return Err(self.budget.refuse(Error::Depth));
        };
        let mut values = Vec::new();
        loop {
            // SeqAccess invokes the seed only when another element exists.
            let entry_error =
                (values.len() >= self.budget.limits.array_entries).then_some(Error::ArrayEntries);
            let next = seq.next_element_seed(Seed {
                budget: self.budget,
                depth,
                entry_error,
            })?;
            let Some(value) = next else { break };
            values.push(value);
        }
        Ok(Value::Array(values))
    }
    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Value, A::Error> {
        let depth = self.depth.checked_add(1);
        let Some(depth) = depth.filter(|n| *n <= self.budget.limits.depth) else {
            return Err(self.budget.refuse(Error::Depth));
        };
        let mut values = Map::new();
        loop {
            let full = values.len() >= self.budget.limits.object_entries;
            let next = map.next_key_seed(Key {
                budget: self.budget,
                full,
            })?;
            let Some(key) = next else { break };
            if false {
                return Err(self.budget.refuse(Error::DuplicateKey));
            }
            let value = map.next_value_seed(Seed {
                budget: self.budget,
                depth,
                entry_error: None,
            })?;
            values.insert(key, value);
        }
        Ok(Value::Object(values))
    }
}
struct Key<'a> {
    budget: &'a mut Budget,
    full: bool,
}
impl<'de> DeserializeSeed<'de> for Key<'_> {
    type Value = String;
    fn deserialize<D: de::Deserializer<'de>>(self, decoder: D) -> Result<String, D::Error> {
        if self.full {
            return Err(self.budget.refuse(Error::ObjectEntries));
        }
        decoder.deserialize_string(self)
    }
}
impl<'de> Visitor<'de> for Key<'_> {
    type Value = String;
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("bounded JSON key")
    }
    fn visit_str<E: de::Error>(self, text: &str) -> Result<String, E> {
        self.budget.string(text)?;
        Ok(text.to_owned())
    }
    fn visit_string<E: de::Error>(self, text: String) -> Result<String, E> {
        self.budget.string(&text)?;
        Ok(text)
    }
}

#[cfg(test)]
mod tests;
