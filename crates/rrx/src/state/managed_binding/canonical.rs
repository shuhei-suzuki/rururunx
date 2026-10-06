//! Complete canonical bodies, retaining original encoded bytes for later CAS.
//! No body, hash or successful parse is a Native allocation/consumption proof.
use anyhow::{Result, ensure};
use serde::{
    Serialize,
    de::{self, DeserializeOwned, DeserializeSeed, MapAccess, SeqAccess, Visitor},
};
use serde_json::{Map, Number, Value};
use sha2::{Digest, Sha256};
use std::{fmt, io::Write};

pub(super) const BODY_BYTES: usize = 8 * 1024 * 1024;
pub(super) const WORKFLOW_DOMAIN: &[u8] = b"rrx.workflow-body-sha256/v1\0";
pub(super) const LEDGER_DOMAIN: &[u8] = b"rrx.workflow-ledger-sha256/v1\0";

/// Complete typed body and exact stored encoding. Deliberately not a grant and
/// not Deserialize: construction always validates the original duplicate keys.
pub(super) struct Body<T> {
    raw: String,
    parsed: T,
    canonical: Vec<u8>,
}
impl<T: DeserializeOwned + Serialize> Body<T> {
    pub(super) fn decode(raw: String, limit: usize) -> Result<Self> {
        let value = decode_value(&raw, limit)?;
        let parsed: T = serde_json::from_value(value.clone())
            .map_err(|_| anyhow::anyhow!("managed body does not match its typed schema"))?;
        let canonical = encode(&value, limit)?;
        ensure!(
            encode(&serde_json::to_value(&parsed)?, limit)? == canonical,
            "managed body typed roundtrip drops or changes fields"
        );
        Ok(Self {
            raw,
            parsed,
            canonical,
        })
    }
    pub(super) fn raw(&self) -> &str {
        &self.raw
    }
    pub(super) fn parsed(&self) -> &T {
        &self.parsed
    }
    pub(super) fn digest(&self, domain: &[u8]) -> String {
        digest(domain, &self.canonical)
    }
}

pub(super) fn digest(domain: &[u8], bytes: &[u8]) -> String {
    let mut hash = Sha256::new();
    hash.update(domain);
    hash.update(bytes);
    format!("{:x}", hash.finalize())
}

/// Unlike the Native wire decoder's 4MiB ceiling, complete stored Workflow
/// bodies have the approved 8MiB budget. This parser has its own local budget;
/// it does not widen any existing wire profile or erase duplicate members.
pub(super) fn decode_value(raw: &str, limit: usize) -> Result<Value> {
    ensure!(
        limit > 0 && limit <= BODY_BYTES && raw.len() <= limit,
        "managed body exceeds complete encoding bound"
    );
    let mut decoder = serde_json::Deserializer::from_str(raw);
    let value = Unique { depth: 0 }
        .deserialize(&mut decoder)
        .map_err(|_| anyhow::anyhow!("managed body JSON is ambiguous or invalid"))?;
    decoder
        .end()
        .map_err(|_| anyhow::anyhow!("managed body has trailing content"))?;
    Ok(value)
}

struct Unique {
    depth: usize,
}
impl<'de> DeserializeSeed<'de> for Unique {
    type Value = Value;
    fn deserialize<D: de::Deserializer<'de>>(
        self,
        decoder: D,
    ) -> std::result::Result<Value, D::Error> {
        if self.depth >= 128 {
            return Err(de::Error::custom("managed JSON depth exceeds bound"));
        }
        decoder.deserialize_any(self)
    }
}
impl<'de> Visitor<'de> for Unique {
    type Value = Value;
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("finite unambiguous managed JSON")
    }
    fn visit_bool<E: de::Error>(self, value: bool) -> std::result::Result<Value, E> {
        Ok(Value::Bool(value))
    }
    fn visit_unit<E: de::Error>(self) -> std::result::Result<Value, E> {
        Ok(Value::Null)
    }
    fn visit_i64<E: de::Error>(self, value: i64) -> std::result::Result<Value, E> {
        Ok(Value::Number(value.into()))
    }
    fn visit_u64<E: de::Error>(self, value: u64) -> std::result::Result<Value, E> {
        Ok(Value::Number(value.into()))
    }
    fn visit_f64<E: de::Error>(self, value: f64) -> std::result::Result<Value, E> {
        Number::from_f64(value)
            .map(Value::Number)
            .ok_or_else(|| E::custom("nonfinite managed number"))
    }
    fn visit_str<E: de::Error>(self, value: &str) -> std::result::Result<Value, E> {
        Ok(Value::String(value.into()))
    }
    fn visit_string<E: de::Error>(self, value: String) -> std::result::Result<Value, E> {
        Ok(Value::String(value))
    }
    fn visit_seq<A: SeqAccess<'de>>(self, mut input: A) -> std::result::Result<Value, A::Error> {
        let mut values = Vec::new();
        while let Some(value) = input.next_element_seed(Unique {
            depth: self.depth + 1,
        })? {
            values.push(value);
        }
        Ok(Value::Array(values))
    }
    fn visit_map<A: MapAccess<'de>>(self, mut input: A) -> std::result::Result<Value, A::Error> {
        let mut values = Map::new();
        while let Some(key) = input.next_key::<String>()? {
            if values.contains_key(&key) {
                return Err(de::Error::custom("duplicate managed JSON member"));
            }
            let value = input.next_value_seed(Unique {
                depth: self.depth + 1,
            })?;
            values.insert(key, value);
        }
        Ok(Value::Object(values))
    }
}

fn ordered(value: &Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut keys = map.keys().collect::<Vec<_>>();
            keys.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));
            Value::Object(
                keys.into_iter()
                    .map(|key| (key.clone(), ordered(&map[key])))
                    .collect(),
            )
        }
        Value::Array(values) => Value::Array(values.iter().map(ordered).collect()),
        scalar => scalar.clone(),
    }
}

pub(super) fn encode(value: &Value, limit: usize) -> Result<Vec<u8>> {
    ensure!(
        limit > 0 && limit <= BODY_BYTES,
        "invalid managed encoding bound"
    );
    struct Limited {
        bytes: Vec<u8>,
        limit: usize,
    }
    impl Write for Limited {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            if bytes.len() > self.limit.saturating_sub(self.bytes.len()) {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "managed complete encoding exceeds bound",
                ));
            }
            self.bytes.extend_from_slice(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut output = Limited {
        bytes: Vec::new(),
        limit,
    };
    serde_json::to_writer(&mut output, &ordered(value))?;
    Ok(output.bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use serde_json::json;

    #[test]
    fn canonical_recipe_keeps_arrays_and_complete_values() {
        let first = decode_value(
            r#"{"z":[{"y":1,"a":"界"},false,null],"a":{"b":2,"a":1}}"#,
            BODY_BYTES,
        )
        .unwrap();
        let reordered = decode_value(
            r#"{"a":{"a":1,"b":2},"z":[{"a":"界","y":1},false,null]}"#,
            BODY_BYTES,
        )
        .unwrap();
        let bytes = encode(&first, BODY_BYTES).unwrap();
        assert_eq!(bytes, encode(&reordered, BODY_BYTES).unwrap());
        assert_eq!(
            String::from_utf8(bytes.clone()).unwrap(),
            r#"{"a":{"a":1,"b":2},"z":[{"a":"界","y":1},false,null]}"#
        );
        let mut altered = first.clone();
        altered["z"].as_array_mut().unwrap().swap(0, 1);
        assert_ne!(
            digest(WORKFLOW_DOMAIN, &bytes),
            digest(WORKFLOW_DOMAIN, &encode(&altered, BODY_BYTES).unwrap())
        );
        altered = first.clone();
        altered["a"]["b"] = json!(3);
        assert_ne!(
            digest(WORKFLOW_DOMAIN, &bytes),
            digest(WORKFLOW_DOMAIN, &encode(&altered, BODY_BYTES).unwrap())
        );
        assert_ne!(
            digest(WORKFLOW_DOMAIN, &bytes),
            digest(LEDGER_DOMAIN, &bytes)
        );
        assert_ne!(
            digest(WORKFLOW_DOMAIN, &bytes),
            digest(b"rrx.workflow-body-sha256/v1", &bytes)
        );
    }

    #[test]
    fn original_bytes_duplicates_and_unknown_fields_are_not_lost() {
        #[derive(Serialize, Deserialize)]
        struct Typed {
            value: u64,
        }
        let original = " { \"value\" : 1 } ".to_owned();
        let body = Body::<Typed>::decode(original.clone(), BODY_BYTES).unwrap();
        assert_eq!(body.raw(), original);
        assert_eq!(body.parsed().value, 1);
        assert_eq!(
            body.digest(WORKFLOW_DOMAIN),
            digest(WORKFLOW_DOMAIN, b"{\"value\":1}")
        );
        for raw in [
            r#"{"value":1,"value":2}"#,
            r#"{"value":1,"\u0076alue":1}"#,
            r#"{"value":1,"ignored":false}"#,
            r#"{"value":1}null"#,
        ] {
            assert!(Body::<Typed>::decode(raw.into(), BODY_BYTES).is_err());
        }
        assert!(decode_value(r#"{"nested":{"x":1,"x":1}}"#, BODY_BYTES).is_err());
    }

    #[test]
    fn inclusive_eight_mib_and_complete_escaped_encoding_bounds() {
        let raw = format!("\"{}\"", "x".repeat(BODY_BYTES - 2));
        let body = Body::<String>::decode(raw.clone(), BODY_BYTES).unwrap();
        assert_eq!(body.raw().len(), BODY_BYTES);
        assert!(Body::<String>::decode(format!(" {raw}"), BODY_BYTES).is_err());
        assert!(encode(&Value::String("\n".repeat(8)), 17).is_err());
        assert_eq!(
            encode(&Value::String("\n".repeat(8)), 18).unwrap().len(),
            18
        );
        assert!(
            decode_value(
                &format!("{}0{}", "[".repeat(128), "]".repeat(128)),
                BODY_BYTES
            )
            .is_err()
        );
    }
}
