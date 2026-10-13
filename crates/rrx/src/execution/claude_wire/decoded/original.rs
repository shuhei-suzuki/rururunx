//! Claude-local exact number provenance. Shared strict_json remains unchanged.
use crate::execution::strict_json::{self, Error, Limits};
use serde_json::{Value, json};

/// The original frame is validated first. The parallel image replaces only
/// numeric tokens with their original ASCII spelling; actual strings remain
/// strings. Consumers must also check the corresponding original Value type.
/// Its size is bounded by frame bytes plus two bytes per numeric node. Parsing
/// it does not impose an additional wire/string budget on legal number tokens.
pub(super) fn decode(bytes: &[u8], limits: Limits) -> Result<(Value, Value), Error> {
    let value = strict_json::decode(bytes, limits)?;
    let mut shadow = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        let start = i;
        match bytes[i] {
            b'"' => {
                i += 1;
                while i < bytes.len() {
                    match bytes[i] {
                        b'\\' => i += 2,
                        b'"' => {
                            i += 1;
                            break;
                        }
                        _ => i += 1,
                    }
                }
                shadow.extend_from_slice(&bytes[start..i]);
            }
            b'-' | b'0'..=b'9' => {
                i += 1;
                while i < bytes.len()
                    && matches!(bytes[i], b'0'..=b'9' | b'.' | b'e' | b'E' | b'+' | b'-')
                {
                    i += 1;
                }
                shadow.push(b'"');
                shadow.extend_from_slice(&bytes[start..i]);
                shadow.push(b'"');
            }
            _ => {
                shadow.push(bytes[i]);
                i += 1;
            }
        }
    }
    let numbers = serde_json::from_slice(&shadow).map_err(|_| Error::InvalidJson)?;
    Ok((value, numbers))
}

struct Decimal {
    negative: bool,
    digits: String,
    exponent: String,
}
impl Decimal {
    // Called only for a numeric lexeme from an already validated JSON frame.
    fn parse(text: &str) -> Self {
        let negative = text.starts_with('-');
        let unsigned = text.strip_prefix('-').unwrap_or(text);
        let (mantissa, exponent) = match unsigned.find(|c| c == 'e' || c == 'E') {
            Some(i) => (&unsigned[..i], &unsigned[i + 1..]),
            None => (unsigned, "0"),
        };
        let fraction = mantissa.find('.').map_or(0, |i| mantissa.len() - i - 1);
        let coefficient = mantissa
            .bytes()
            .filter(|b| *b != b'.')
            .map(char::from)
            .collect::<String>();
        let significant = coefficient.trim_start_matches('0');
        if significant.is_empty() {
            return Self {
                negative,
                digits: "0".into(),
                exponent: "0".into(),
            };
        }
        let digits = significant.trim_end_matches('0');
        let shift = (significant.len() - digits.len()) as i64 - fraction as i64;
        Self {
            negative,
            digits: digits.into(),
            exponent: add_signed(&normalize_signed(exponent), &shift.to_string()),
        }
    }
    fn uint(&self) -> Option<u64> {
        if self.digits == "0" {
            return Some(0);
        }
        if self.negative || self.exponent.starts_with('-') {
            return None;
        }
        let exponent = self.exponent.parse::<usize>().ok()?;
        if self.digits.len().checked_add(exponent)? > 20 {
            return None;
        }
        let mut n = 0_u64;
        for digit in self.digits.bytes() {
            n = n.checked_mul(10)?.checked_add(u64::from(digit - b'0'))?;
        }
        for _ in 0..exponent {
            n = n.checked_mul(10)?;
        }
        Some(n)
    }
}

fn normalize_signed(text: &str) -> String {
    let negative = text.starts_with('-');
    let magnitude = text
        .strip_prefix('-')
        .or_else(|| text.strip_prefix('+'))
        .unwrap_or(text);
    let magnitude = magnitude.trim_start_matches('0');
    if magnitude.is_empty() {
        "0".into()
    } else if negative {
        format!("-{magnitude}")
    } else {
        magnitude.into()
    }
}
fn parts(text: &str) -> (bool, &str) {
    match text.strip_prefix('-') {
        Some(magnitude) => (true, magnitude),
        None => (false, text),
    }
}
fn add_magnitude(a: &str, b: &str) -> String {
    let mut a = a.bytes().rev();
    let mut b = b.bytes().rev();
    let mut carry = 0;
    let mut out = Vec::new();
    loop {
        let x = a.next();
        let y = b.next();
        if x.is_none() && y.is_none() && carry == 0 {
            break;
        }
        let sum = x.map_or(0, |n| n - b'0') + y.map_or(0, |n| n - b'0') + carry;
        out.push(b'0' + sum % 10);
        carry = sum / 10;
    }
    out.into_iter().rev().map(char::from).collect()
}
fn subtract_magnitude(a: &str, b: &str) -> String {
    // a >= b; both inputs are normalized decimal magnitudes.
    let mut b = b.bytes().rev();
    let mut borrow = 0_i16;
    let mut out = Vec::new();
    for x in a.bytes().rev() {
        let mut digit = i16::from(x - b'0') - borrow - i16::from(b.next().map_or(0, |n| n - b'0'));
        borrow = if digit < 0 {
            digit += 10;
            1
        } else {
            0
        };
        out.push(b'0' + digit as u8);
    }
    let out = out.into_iter().rev().map(char::from).collect::<String>();
    let out = out.trim_start_matches('0');
    if out.is_empty() {
        "0".into()
    } else {
        out.into()
    }
}
fn add_signed(a: &str, b: &str) -> String {
    let (an, am) = parts(a);
    let (bn, bm) = parts(b);
    let (negative, magnitude) = if an == bn {
        (an, add_magnitude(am, bm))
    } else {
        let order = am.len().cmp(&bm.len()).then_with(|| am.cmp(bm));
        if order.is_lt() {
            (bn, subtract_magnitude(bm, am))
        } else {
            (an, subtract_magnitude(am, bm))
        }
    };
    if magnitude == "0" {
        magnitude
    } else if negative {
        format!("-{magnitude}")
    } else {
        magnitude
    }
}

pub(super) fn uint(value: &Value, numbers: &Value) -> Option<u64> {
    if !value.is_number() {
        return None;
    }
    Decimal::parse(numbers.as_str()?).uint()
}

/// Type tags prevent numeric-token strings from colliding with actual strings
/// or user objects. Key order and normalized top-level UUID remain semantic.
/// Preserve the previous integer/float representation distinction and float
/// signed zero, while never identifying distinct exact numbers by f64 rounding.
pub(super) fn canonical(value: &Value, numbers: &Value) -> Result<Value, Error> {
    Ok(match value {
        Value::Null => json!(["null"]),
        Value::Bool(value) => json!(["bool", value]),
        Value::String(value) => json!(["string", value]),
        Value::Number(number) => {
            let number_text = numbers.as_str().ok_or(Error::InvalidJson)?;
            let decimal = Decimal::parse(number_text);
            json!([
                "number",
                number.is_f64(),
                decimal.negative,
                decimal.digits,
                decimal.exponent
            ])
        }
        Value::Array(values) => {
            let original = numbers.as_array().ok_or(Error::InvalidJson)?;
            if values.len() != original.len() {
                return Err(Error::InvalidJson);
            }
            let values = values
                .iter()
                .zip(original)
                .map(|(v, n)| canonical(v, n))
                .collect::<Result<Vec<_>, _>>()?;
            json!(["array", values])
        }
        Value::Object(object) => {
            let original = numbers.as_object().ok_or(Error::InvalidJson)?;
            let mut keys = object.keys().collect::<Vec<_>>();
            keys.sort();
            let mut entries = Vec::with_capacity(keys.len());
            for key in keys {
                let n = original.get(key).ok_or(Error::InvalidJson)?;
                entries.push(json!([key, canonical(&object[key], n)?]));
            }
            json!(["object", entries])
        }
    })
}
