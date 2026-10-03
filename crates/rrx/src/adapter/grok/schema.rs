//! Deliberately bounded JSON Schema subset shared by dispatch and local validation.
//! Unsupported semantics are rejected before native inference, never silently ignored.
use super::{AdapterResult, ErrorKind, failure};
use serde_json::Value;

const KEYS: &[&str] = &[
    "type",
    "properties",
    "required",
    "additionalProperties",
    "items",
    "enum",
    "minLength",
    "maxLength",
    "minItems",
    "maxItems",
    "minimum",
    "maximum",
    "description",
    "title",
];

pub(super) fn check(schema: &Value) -> AdapterResult<()> {
    if serde_json::to_vec(schema).map_or(true, |v| v.len() > 16_384) {
        return Err(failure(
            ErrorKind::InvalidInput,
            "output schema exceeds budget",
        ));
    }
    inspect(schema, 0)
}
fn inspect(schema: &Value, depth: usize) -> AdapterResult<()> {
    let invalid = || {
        failure(
            ErrorKind::InvalidInput,
            "unsupported or malformed bounded output schema",
        )
    };
    let object = schema.as_object().ok_or_else(invalid)?;
    if depth > 16 || object.keys().any(|key| !KEYS.contains(&key.as_str())) {
        return Err(invalid());
    }
    let kind = object
        .get("type")
        .and_then(Value::as_str)
        .ok_or_else(invalid)?;
    if ![
        "object", "array", "string", "integer", "number", "boolean", "null",
    ]
    .contains(&kind)
    {
        return Err(invalid());
    }
    for key in ["description", "title"] {
        if object.get(key).is_some_and(|v| !v.is_string()) {
            return Err(invalid());
        }
    }
    if let Some(values) = object.get("enum") {
        if values
            .as_array()
            .is_none_or(|values| values.is_empty() || values.len() > 128)
        {
            return Err(invalid());
        }
    }
    for key in ["minLength", "maxLength", "minItems", "maxItems"] {
        if let Some(value) = object.get(key) {
            if value.as_u64().is_none()
                || !(if key.ends_with("Length") {
                    kind == "string"
                } else {
                    kind == "array"
                })
            {
                return Err(invalid());
            }
        }
    }
    for key in ["minimum", "maximum"] {
        if object
            .get(key)
            .is_some_and(|v| !v.is_number() || !["number", "integer"].contains(&kind))
        {
            return Err(invalid());
        }
    }
    if let Some(properties) = object.get("properties") {
        if kind != "object" {
            return Err(invalid());
        }
        let properties = properties.as_object().ok_or_else(invalid)?;
        if properties.len() > 128 {
            return Err(invalid());
        }
        for property in properties.values() {
            inspect(property, depth + 1)?;
        }
    }
    if let Some(required) = object.get("required") {
        if kind != "object"
            || required
                .as_array()
                .is_none_or(|a| a.len() > 128 || a.iter().any(|v| !v.is_string()))
        {
            return Err(invalid());
        }
    }
    if object
        .get("additionalProperties")
        .is_some_and(|v| kind != "object" || !v.is_boolean())
    {
        return Err(invalid());
    }
    if let Some(items) = object.get("items") {
        if kind != "array" {
            return Err(invalid());
        }
        inspect(items, depth + 1)?;
    }
    Ok(())
}

pub(super) fn validate(schema: &Value, value: &Value) -> AdapterResult<()> {
    if matches_value(schema, value, 0) {
        Ok(())
    } else {
        Err(failure(
            ErrorKind::ParseFailure,
            "native structured output violates requested schema",
        ))
    }
}
fn matches_value(schema: &Value, value: &Value, depth: usize) -> bool {
    if depth > 16 {
        return false;
    }
    let type_matches = match schema["type"].as_str() {
        Some("object") => value.is_object(),
        Some("array") => value.is_array(),
        Some("string") => value.is_string(),
        Some("integer") => value.as_i64().is_some() || value.as_u64().is_some(),
        Some("number") => value.is_number(),
        Some("boolean") => value.is_boolean(),
        Some("null") => value.is_null(),
        _ => false,
    };
    if !type_matches
        || schema["enum"]
            .as_array()
            .is_some_and(|a| !a.contains(value))
    {
        return false;
    }
    let length = if let Some(s) = value.as_str() {
        Some((s.chars().count() as u64, "minLength", "maxLength"))
    } else {
        value
            .as_array()
            .map(|a| (a.len() as u64, "minItems", "maxItems"))
    };
    if let Some((length, min, max)) = length {
        if schema[min].as_u64().is_some_and(|v| length < v)
            || schema[max].as_u64().is_some_and(|v| length > v)
        {
            return false;
        }
    }
    if let Some(number) = value.as_f64() {
        if schema["minimum"].as_f64().is_some_and(|v| number < v)
            || schema["maximum"].as_f64().is_some_and(|v| number > v)
        {
            return false;
        }
    }
    if let Some(object) = value.as_object() {
        if schema["required"].as_array().is_some_and(|a| {
            a.iter()
                .any(|v| !object.contains_key(v.as_str().unwrap_or("")))
        }) {
            return false;
        }
        for (key, value) in object {
            if let Some(property) = schema["properties"].get(key) {
                if !matches_value(property, value, depth + 1) {
                    return false;
                }
            } else if schema["additionalProperties"] == false {
                return false;
            }
        }
    }
    if let Some(values) = value.as_array() {
        if let Some(items) = schema.get("items") {
            if values.iter().any(|v| !matches_value(items, v, depth + 1)) {
                return false;
            }
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn output_contract_validates_nested_required_enum_extras_and_numeric_bounds() {
        let schema = json!({"type":"object","properties":{"verdict":{"type":"string","enum":["DENY"]},"facts":{"type":"array","minItems":1,"maxItems":2,"items":{"type":"integer","minimum":1,"maximum":3}},"reason":{"type":"string","minLength":2,"maxLength":3}},"required":["verdict","facts","reason"],"additionalProperties":false});
        check(&schema).unwrap();
        validate(
            &schema,
            &json!({"verdict":"DENY","facts":[2],"reason":"ok"}),
        )
        .unwrap();
        for invalid in [
            json!({"verdict":"ALLOW","facts":[2],"reason":"ok"}),
            json!({"verdict":"DENY","facts":[2]}),
            json!({"verdict":"DENY","facts":[2],"reason":"ok","extra":true}),
            json!({"verdict":"DENY","facts":[0],"reason":"ok"}),
            json!({"verdict":"DENY","facts":[],"reason":"ok"}),
            json!({"verdict":"DENY","facts":[2],"reason":"long"}),
        ] {
            assert!(validate(&schema, &invalid).is_err(), "{invalid}");
        }
        assert!(check(&json!({"type":"object","$ref":"https://foreign.invalid/schema"})).is_err());
        assert!(check(&json!({"type":"string","properties":{}})).is_err());
    }
}
