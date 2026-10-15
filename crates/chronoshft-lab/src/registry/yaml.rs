//! Parse registry YAML with duplicate checks and PyYAML-compatible scalar rules.
//! Dates must be quoted JSON strings. Merge keys/custom tags are not supported
//! by the reference UniqueKeyLoader either. Recursive aliases are rejected.
use num_bigint::BigInt;
use regex::Regex;
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::sync::LazyLock;
use yaml_rust2::parser::{Event, Parser};
use yaml_rust2::scanner::TScalarStyle;

static INT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^[-+]?(?:0b[0-1_]+|0[0-7_]+|(?:0|[1-9][0-9_]*)|0x[0-9a-fA-F_]+|[1-9][0-9_]*(?::[0-5]?[0-9])+)$").unwrap()
});
static FLOAT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(?:[-+]?[0-9][0-9_]*\.[0-9_]*(?:[eE][-+][0-9]+)?|\.[0-9][0-9_]*(?:[eE][-+][0-9]+)?|[-+]?[0-9][0-9_]*(?::[0-5]?[0-9])+\.[0-9_]*|[-+]?\.(?:inf|Inf|INF)|\.(?:nan|NaN|NAN))$").unwrap()
});
static TIMESTAMP: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?x)^(?:[0-9]{4}-[0-9]{2}-[0-9]{2}|[0-9]{4}-[0-9]{1,2}-[0-9]{1,2}(?:[Tt]|[ \t]+)[0-9]{1,2}:[0-9]{2}:[0-9]{2}(?:\.[0-9]*)?(?:[ \t]*(?:Z|[-+][0-9]{1,2}(?::[0-9]{2})?))?)$").unwrap()
});

fn integer(text: &str) -> Result<Value, String> {
    let text = text.replace('_', "");
    let negative = text.starts_with('-');
    let digits = if text.starts_with(['+', '-']) {
        &text[1..]
    } else {
        &text
    };
    let mut number = if digits.contains(':') {
        let mut n = BigInt::from(0);
        for piece in digits.split(':') {
            n = n * 60 + BigInt::parse_bytes(piece.as_bytes(), 10).ok_or("invalid YAML integer")?;
        }
        n
    } else {
        let (radix, digits) = if let Some(d) = digits.strip_prefix("0b") {
            (2, d)
        } else if let Some(d) = digits.strip_prefix("0x") {
            (16, d)
        } else if let Some(d) = digits.strip_prefix("0o") {
            (8, d)
        } else if digits.starts_with('0') && digits.len() > 1 {
            (8, digits)
        } else {
            (10, digits)
        };
        BigInt::parse_bytes(digits.as_bytes(), radix).ok_or("invalid YAML integer")?
    };
    if negative {
        number = -number;
    }
    serde_json::from_str(&number.to_string()).map_err(|e| e.to_string())
}

fn scalar(text: String, style: TScalarStyle, tag: Option<&str>) -> Result<Value, String> {
    if tag == Some("str") {
        return Ok(Value::String(text));
    }
    if tag.is_none() && style != TScalarStyle::Plain {
        return Ok(Value::String(text));
    }
    let boolean = match text.as_str() {
        "yes" | "Yes" | "YES" | "true" | "True" | "TRUE" | "on" | "On" | "ON" => Some(true),
        "no" | "No" | "NO" | "false" | "False" | "FALSE" | "off" | "Off" | "OFF" => Some(false),
        _ => None,
    };
    let inferred = if matches!(text.as_str(), "" | "~" | "null" | "Null" | "NULL") {
        "null"
    } else if boolean.is_some() {
        "bool"
    } else if INT.is_match(&text) {
        "int"
    } else if FLOAT.is_match(&text) {
        "float"
    } else if TIMESTAMP.is_match(&text) {
        "timestamp"
    } else {
        "str"
    };
    match tag.unwrap_or(inferred) {
        "null" => Ok(Value::Null),
        "bool" => (if tag == Some("bool") {
            match text.to_ascii_lowercase().as_str() {
                "yes" | "true" | "on" => Some(true),
                "no" | "false" | "off" => Some(false),
                _ => None,
            }
        } else {
            boolean
        })
        .map(Value::Bool)
        .ok_or_else(|| "invalid YAML boolean".into()),
        "int" => integer(&text),
        "float" => {
            let clean = text.replace('_', "");
            let negative = clean.starts_with('-');
            let digits = if clean.starts_with(['+', '-']) {
                &clean[1..]
            } else {
                &clean
            };
            let number = if digits.contains(':') {
                let mut n = 0.0;
                let mut base = 1.0;
                for piece in digits.rsplit(':') {
                    n += base * piece.parse::<f64>().map_err(|e| e.to_string())?;
                    base *= 60.0;
                }
                if negative {
                    -n
                } else {
                    n
                }
            } else {
                (if negative { -1.0 } else { 1.0 })
                    * digits
                        .parse::<f64>()
                        .map_err(|_| "non-finite or invalid YAML float")?
            };
            serde_json::Number::from_f64(number)
                .map(Value::Number)
                .ok_or_else(|| "non-finite numbers are not JSON-compatible".into())
        }
        "timestamp" => Err("YAML date/timestamp is not a JSON string; quote date values".into()),
        "str" => Ok(Value::String(text)),
        other => Err(format!("unsupported YAML tag: {other}")),
    }
}

struct Reader {
    events: Vec<Event>,
    position: usize,
    anchors: HashMap<usize, Value>,
    expanded: usize,
    mkdocs_formatter: bool,
}
impl Reader {
    fn next(&mut self) -> Result<Event, String> {
        let event = self
            .events
            .get(self.position)
            .cloned()
            .ok_or("unexpected end of YAML")?;
        self.position += 1;
        Ok(event)
    }
    fn node(&mut self, depth: usize) -> Result<Value, String> {
        self.expanded += 1;
        if depth > 128 || self.expanded > 100_000 {
            return Err("YAML nesting/expansion limit exceeded".into());
        }
        let (anchor, value) = match self.next()? {
            Event::Scalar(text, style, anchor, tag) => {
                // MkDocs owns this existing callable tag. The link checker only
                // needs build paths and keeps the formatter name inert. Registry
                // parsing never enables this exception; top-level settings may
                // not use callable tags.
                if depth > 1
                    && self.mkdocs_formatter
                    && tag.as_ref().is_some_and(|t| {
                        t.handle == "tag:yaml.org,2002:"
                            && t.suffix == "python/name:pymdownx.superfences.fence_code_format"
                    })
                {
                    return Ok(Value::String(text));
                }
                let kind = if let Some(ref tag) = tag {
                    if tag.handle != "tag:yaml.org,2002:" {
                        return Err("unsupported YAML tag".into());
                    }
                    Some(tag.suffix.as_str())
                } else {
                    None
                };
                (anchor, scalar(text, style, kind)?)
            }
            Event::SequenceStart(anchor, tag) => {
                if tag.is_some_and(|t| t.handle != "tag:yaml.org,2002:" || t.suffix != "seq") {
                    return Err("unsupported YAML tag".into());
                }
                let mut values = Vec::new();
                while self.events.get(self.position) != Some(&Event::SequenceEnd) {
                    values.push(self.node(depth + 1)?);
                }
                self.next()?;
                (anchor, Value::Array(values))
            }
            Event::MappingStart(anchor, tag) => {
                if tag.is_some_and(|t| t.handle != "tag:yaml.org,2002:" || t.suffix != "map") {
                    return Err("unsupported YAML tag".into());
                }
                let mut values = Map::new();
                while self.events.get(self.position) != Some(&Event::MappingEnd) {
                    let key = self
                        .node(depth + 1)?
                        .as_str()
                        .ok_or("YAML object keys must be strings")?
                        .to_owned();
                    if key == "<<" {
                        return Err("YAML merge keys are unsupported".into());
                    }
                    if values.contains_key(&key) {
                        return Err(format!("duplicate YAML key: {key}"));
                    }
                    let value = self.node(depth + 1)?;
                    values.insert(key, value);
                }
                self.next()?;
                (anchor, Value::Object(values))
            }
            Event::Alias(id) => {
                let value = self
                    .anchors
                    .get(&id)
                    .ok_or("undefined or recursive YAML alias")?;
                // Count expanded alias content before cloning, bounding expansion.
                fn size(v: &Value) -> usize {
                    match v {
                        Value::Array(a) => 1 + a.iter().map(size).sum::<usize>(),
                        Value::Object(o) => 1 + o.values().map(size).sum::<usize>(),
                        _ => 1,
                    }
                }
                self.expanded += size(value);
                if self.expanded > 100_000 {
                    return Err("YAML expansion limit exceeded".into());
                }
                return Ok(value.clone());
            }
            _ => return Err("expected a YAML value".into()),
        };
        if anchor != 0 {
            self.anchors.insert(anchor, value.clone());
        }
        Ok(value)
    }
}

pub fn parse(text: &str) -> Result<Value, String> {
    parse_mode(text, false)
}

/// Parse the repository's MkDocs configuration without loading its Python
/// formatter. The exception is narrowly scoped and never applies to registries.
pub(crate) fn parse_mkdocs(text: &str) -> Result<Value, String> {
    parse_mode(text, true)
}

fn parse_mode(text: &str, mkdocs_formatter: bool) -> Result<Value, String> {
    let mut parser = Parser::new_from_str(text);
    let mut events = Vec::new();
    loop {
        let (event, _) = parser.next_token().map_err(|e| e.to_string())?;
        let done = event == Event::StreamEnd;
        events.push(event);
        if done {
            break;
        }
    }
    let mut reader = Reader {
        events,
        position: 0,
        anchors: HashMap::new(),
        expanded: 0,
        mkdocs_formatter,
    };
    if reader.next()? != Event::StreamStart {
        return Err("missing YAML stream".into());
    }
    if reader.events.get(reader.position) == Some(&Event::StreamEnd) {
        return Ok(Value::Null);
    }
    if reader.next()? != Event::DocumentStart {
        return Err("missing YAML document".into());
    }
    let value = reader.node(0)?;
    if reader.next()? != Event::DocumentEnd || reader.next()? != Event::StreamEnd {
        return Err("expected exactly one YAML document".into());
    }
    Ok(value)
}
