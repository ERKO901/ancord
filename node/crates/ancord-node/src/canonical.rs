//! Kanonické JSON: RFC 8785 (JCS) omezené na celá čísla (`spec/PROTOCOL.md` §1.2).

use serde_json::Value;

const MAX_SAFE: i64 = 9_007_199_254_740_991;

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum CanonicalError {
    #[error("canonical JSON allows safe integers only")]
    NotSafeInteger,
}

/// Serializuje hodnotu tak, aby byla bajt po bajtu shodná s Workerem.
pub fn canonical_json(value: &Value) -> Result<String, CanonicalError> {
    let mut out = String::new();
    write_value(value, &mut out)?;
    Ok(out)
}

pub fn canonical_bytes(value: &Value) -> Result<Vec<u8>, CanonicalError> {
    canonical_json(value).map(String::into_bytes)
}

fn write_value(value: &Value, out: &mut String) -> Result<(), CanonicalError> {
    match value {
        Value::Null => out.push_str("null"),
        Value::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        Value::Number(n) => {
            let i = n.as_i64().ok_or(CanonicalError::NotSafeInteger)?;
            if !(-MAX_SAFE..=MAX_SAFE).contains(&i) {
                return Err(CanonicalError::NotSafeInteger);
            }
            out.push_str(&i.to_string());
        }
        Value::String(s) => write_string(s, out),
        Value::Array(items) => {
            out.push('[');
            for (k, item) in items.iter().enumerate() {
                if k > 0 {
                    out.push(',');
                }
                write_value(item, out)?;
            }
            out.push(']');
        }
        Value::Object(map) => {
            // pořadí podle kódových jednotek UTF-16, ne podle bajtů UTF-8
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort_by(|a, b| a.encode_utf16().cmp(b.encode_utf16()));
            out.push('{');
            for (k, key) in keys.into_iter().enumerate() {
                if k > 0 {
                    out.push(',');
                }
                write_string(key, out);
                out.push(':');
                if let Some(v) = map.get(key) {
                    write_value(v, out)?;
                }
            }
            out.push('}');
        }
    }
    Ok(())
}

/// Escapování shodné s `JSON.stringify`.
fn write_string(s: &str, out: &mut String) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\u{08}' => out.push_str("\\b"),
            '\u{0c}' => out.push_str("\\f"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn rejects_floats_and_big_numbers() {
        assert!(canonical_json(&json!(1.5)).is_err());
        assert!(canonical_json(&json!(9_007_199_254_740_992_i64)).is_err());
        assert!(canonical_json(&json!(u64::MAX)).is_err());
    }

    #[test]
    fn utf16_key_order() {
        // U+1F600 (surrogáty D83D…) se řadí před U+FF5E, v UTF-8 by byl až za ním
        let v = json!({"\u{ff5e}": 1, "\u{1f600}": 2});
        assert_eq!(canonical_json(&v).unwrap_or_default(), "{\"\u{1f600}\":2,\"\u{ff5e}\":1}");
    }

    #[test]
    fn control_chars_escaped_like_js() {
        let v = json!("a\u{1f}\u{7f}");
        assert_eq!(canonical_json(&v).unwrap_or_default(), "\"a\\u001f\u{7f}\"");
    }
}
