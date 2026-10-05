use super::*;
use serde_json::json;

fn parse(bytes: &[u8]) -> Result<Value, Error> {
    decode(bytes, Limits::CEILINGS)
}

#[test]
fn complete_nested_values_and_scalar_forms() {
    let text = br#" {"a":[null,true,false,-4,18446744073709551615,1.25,"hello",{}],"b":[]} "#;
    assert_eq!(
        parse(text).unwrap(),
        serde_json::from_slice::<Value>(text).unwrap()
    );
    for text in [
        "null", "true", "false", "-1", "0", "1.25e2", "\"text\"", "{}", "[]",
    ] {
        assert_eq!(
            parse(text.as_bytes()).unwrap(),
            serde_json::from_str::<Value>(text).unwrap()
        );
    }
}

#[test]
fn duplicate_keys_in_every_object_and_escaped_spellings_refuse() {
    for text in [
        r#"{"verdict":"REQUEST_CHANGES","verdict":"APPROVE"}"#,
        r#"{"data":{"x":1,"x":2}}"#,
        r#"[{"x":1,"x":1}]"#,
        r#"{"a":{"b":[{"deep":null,"deep":true}]}}"#,
        r#"{"a":1,"\u0061":2}"#,
        r#"{"\u00e9":1,"é":2}"#,
        r#"{"":1,"":2}"#,
    ] {
        assert_eq!(parse(text.as_bytes()), Err(Error::DuplicateKey));
    }
    assert_eq!(
        parse(br#"{"a":1,"nested":{"a":2}}"#).unwrap(),
        json!({"a":1,"nested":{"a":2}})
    );
}

#[test]
fn malformed_input_and_prefix_recovery_are_rejected() {
    for text in [
        "",
        " ",
        "NaN",
        "Infinity",
        "-Infinity",
        "1e400",
        "01",
        "+1",
        "{",
        "[1,]",
        "{\"x\":1,}",
        "null true",
        "{}suffix",
        "```json\n{}\n```",
        "answer: {}",
        "\"\\uD800\"",
        "\"a\n\"",
    ] {
        assert_eq!(
            parse(text.as_bytes()),
            Err(Error::InvalidJson),
            "invalid input accepted"
        );
    }
    assert_eq!(parse(&[b'"', 0xff, b'"']), Err(Error::InvalidJson));
    assert_eq!(parse(&[0xff]), Err(Error::InvalidJson));
}

#[test]
fn frame_nodes_and_entry_bounds_are_inclusive() {
    let mut limits = Limits::CEILINGS;
    limits.frame_bytes = 4;
    assert_eq!(decode(b"null", limits).unwrap(), Value::Null);
    assert_eq!(decode(b"null ", limits), Err(Error::FrameBytes));
    limits = Limits::CEILINGS;
    limits.nodes = 3;
    assert_eq!(decode(b"[0,1]", limits).unwrap(), json!([0, 1]));
    assert_eq!(decode(b"[0,1,2]", limits), Err(Error::Nodes));
    assert_eq!(
        decode(br#"{"a":0,"b":1}"#, limits).unwrap(),
        json!({"a":0,"b":1})
    );
    limits.nodes = 1;
    assert_eq!(decode(b"{}", limits).unwrap(), json!({}));
    assert_eq!(decode(b"[]", limits).unwrap(), json!([]));
    assert_eq!(decode(br#"{"a":0}"#, limits), Err(Error::Nodes));
    limits = Limits::CEILINGS;
    limits.array_entries = 2;
    assert_eq!(decode(b"[0,1]", limits).unwrap(), json!([0, 1]));
    assert_eq!(decode(b"[0,1,2]", limits), Err(Error::ArrayEntries));
    limits.object_entries = 2;
    assert_eq!(
        decode(br#"{"a":0,"b":1}"#, limits).unwrap(),
        json!({"a":0,"b":1})
    );
    assert_eq!(
        decode(br#"{"a":0,"b":1,"c":2}"#, limits),
        Err(Error::ObjectEntries)
    );
    assert_eq!(
        decode(br#"[{"a":0,"b":1},{"c":0,"d":1}]"#, limits).unwrap(),
        json!([{"a":0,"b":1},{"c":0,"d":1}])
    );
}

#[test]
fn container_depth_counts_objects_and_arrays_with_exact_ceiling() {
    for limit in [1, 2, 32] {
        let mut limits = Limits::CEILINGS;
        limits.depth = limit;
        let exact = format!("{}0{}", "[".repeat(limit), "]".repeat(limit));
        assert!(decode(exact.as_bytes(), limits).is_ok());
        let excess = format!("[{exact}]");
        assert_eq!(decode(excess.as_bytes(), limits), Err(Error::Depth));
    }
    let mut limits = Limits::CEILINGS;
    limits.depth = 2;
    assert!(decode(br#"{"a":[]}"#, limits).is_ok());
    assert_eq!(decode(br#"{"a":[{}]}"#, limits), Err(Error::Depth));
}

#[test]
fn decoded_utf8_keys_and_strings_share_byte_budgets() {
    let mut limits = Limits::CEILINGS;
    limits.string_bytes = 2;
    limits.total_string_bytes = 4;
    for text in [r#"{"é":"é"}"#, r#"{"\u00e9":"\u00e9"}"#] {
        assert_eq!(decode(text.as_bytes(), limits).unwrap(), json!({"é":"é"}));
    }
    assert_eq!(decode("\"éa\"".as_bytes(), limits), Err(Error::StringBytes));
    assert_eq!(decode(br#"{"abc":0}"#, limits), Err(Error::StringBytes));
    assert_eq!(
        decode("[\"é\",\"é\",\"a\"]".as_bytes(), limits),
        Err(Error::TotalStringBytes)
    );
    assert_eq!(
        decode(br#"{"aa":"bb","c":0}"#, limits),
        Err(Error::TotalStringBytes)
    );
    limits.string_bytes = 1;
    limits.total_string_bytes = 1;
    assert_eq!(decode(b"\"a\"", limits).unwrap(), json!("a"));
    assert_eq!(decode(br#"{"a":0}"#, limits).unwrap(), json!({"a":0}));
    assert_eq!(
        decode(br#"{"a":"b"}"#, limits),
        Err(Error::TotalStringBytes)
    );
}

#[test]
fn every_invalid_profile_refuses_before_parsing() {
    for i in 0..7 {
        for excess in [false, true] {
            let mut limits = Limits::CEILINGS;
            let field = match i {
                0 => &mut limits.frame_bytes,
                1 => &mut limits.depth,
                2 => &mut limits.nodes,
                3 => &mut limits.string_bytes,
                4 => &mut limits.total_string_bytes,
                5 => &mut limits.object_entries,
                _ => &mut limits.array_entries,
            };
            *field = if excess {
                field.checked_add(1).unwrap()
            } else {
                0
            };
            assert_eq!(
                decode(b"not JSON secret", limits),
                Err(Error::InvalidLimits)
            );
        }
    }
}

#[test]
fn errors_never_render_source_keys_or_parser_payloads() {
    let secret = "fixture-sensitive-text";
    for text in [
        format!(r#"{{"{secret}":0,"{secret}":1}}"#),
        format!(r#"{{"field":"{secret}""#),
    ] {
        let error = parse(text.as_bytes()).unwrap_err();
        assert!(!format!("{error:?} {error}").contains(secret));
        assert!(matches!(error, Error::DuplicateKey | Error::InvalidJson));
    }
}
