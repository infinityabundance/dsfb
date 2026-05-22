//! Minimal TOML-subset parser for the corpus source-ingestion format.
//!
//! T.2 ships a hand-rolled parser instead of pulling in the `toml`
//! crate, mirroring the dsfb workspace's zero-dep posture. The
//! subset we accept is exactly what the corpus format needs:
//!
//! - `# comment` to end of line.
//! - Blank lines ignored.
//! - `[[detector]]` opens a new top-level array element.
//! - `[detector.<name>]` opens a single named subtable of the
//!   current top-level element.
//! - `[[detector.<name>]]` opens a new element of the named
//!   array-of-subtables on the current top-level element.
//! - `key = value` assignment within the current section.
//! - Values: double-quoted strings (with `\"`, `\\`, `\n` escapes),
//!   decimal integers (with optional `-` sign), `true` / `false`,
//!   and homogeneous arrays of those atoms.
//!
//! Output is a `Vec<DetectorRecord>` where each record carries
//! flat `fields`, named `subtables`, and named `array_subtables`.
//! The loader in [`crate::loader`] walks this AST to construct
//! [`crate::types::LiteratureDetector`]-shaped records.

extern crate alloc;
use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

/// Atomic value the parser produces.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    /// Double-quoted string (escapes already decoded).
    String(String),
    /// Decimal integer.
    Int(i64),
    /// `true` or `false`.
    Bool(bool),
    /// Homogeneous array of atoms (or empty).
    Array(Vec<Value>),
}

/// One detector record in the TOML AST.
///
/// `fields` are direct `key = value` assignments under `[[detector]]`.
/// `subtables` are `[detector.<name>]` sections (one entry per
/// distinct name). `array_subtables` are `[[detector.<name>]]`
/// arrays of subtables (e.g. `source_refs`).
#[derive(Debug, Clone, Default)]
pub struct DetectorRecord {
    /// Direct key/value pairs assigned at the top of the record.
    pub fields: BTreeMap<String, Value>,
    /// Single named subtable per name (e.g. `parameter_bounds`,
    /// `genealogy`, `constitution_compliance`).
    pub subtables: BTreeMap<String, BTreeMap<String, Value>>,
    /// Named array of subtables (e.g. `source_refs`).
    pub array_subtables: BTreeMap<String, Vec<BTreeMap<String, Value>>>,
}

/// Parse error with line number.
#[derive(Debug, Clone)]
pub struct ParseError {
    /// 1-based line number where the error was detected.
    pub line: usize,
    /// Human-readable error message.
    pub message: String,
}

impl ParseError {
    /// Format as a single-line diagnostic string.
    #[must_use]
    pub fn display(&self) -> String {
        format!("line {}: {}", self.line, self.message)
    }
}

/// Parse the input source into the detector AST.
///
/// # Errors
/// Returns the first parse error encountered. The parser does not
/// recover; it stops at the first syntactic issue.
pub fn parse_detectors(input: &str) -> Result<Vec<DetectorRecord>, ParseError> {
    let mut records: Vec<DetectorRecord> = Vec::new();
    let mut cursor = Cursor::TopLevel;
    for (idx, raw_line) in input.lines().enumerate() {
        let line_no = idx + 1;
        let stripped = strip_comment(raw_line).trim();
        if stripped.is_empty() {
            continue;
        }
        if let Some(inner) = section_header(stripped, "[[", "]]") {
            cursor = handle_array_section(&mut records, &cursor, inner, line_no)?;
            continue;
        }
        if let Some(inner) = section_header(stripped, "[", "]") {
            cursor = handle_subtable_section(&mut records, &cursor, inner, line_no)?;
            continue;
        }
        let (key, value) = parse_assignment(stripped, line_no)?;
        assign_into_cursor(&mut records, &cursor, key, value, line_no)?;
    }
    Ok(records)
}

fn handle_array_section(
    records: &mut Vec<DetectorRecord>,
    cursor: &Cursor,
    inner: &str,
    line_no: usize,
) -> Result<Cursor, ParseError> {
    let parts: Vec<&str> = inner.split('.').collect();
    match parts.as_slice() {
        ["detector"] => {
            records.push(DetectorRecord::default());
            Ok(Cursor::Detector(records.len() - 1))
        }
        ["detector", name] => {
            let det_idx = current_detector_index(cursor, line_no)?;
            let vec = records[det_idx]
                .array_subtables
                .entry((*name).to_string())
                .or_default();
            vec.push(BTreeMap::new());
            Ok(Cursor::ArraySubtable(
                det_idx,
                (*name).to_string(),
                vec.len() - 1,
            ))
        }
        _ => Err(ParseError {
            line: line_no,
            message: format!("unsupported array-table header `[[{inner}]]`"),
        }),
    }
}

fn handle_subtable_section(
    records: &mut [DetectorRecord],
    cursor: &Cursor,
    inner: &str,
    line_no: usize,
) -> Result<Cursor, ParseError> {
    let parts: Vec<&str> = inner.split('.').collect();
    match parts.as_slice() {
        ["detector", name] => {
            let det_idx = current_detector_index(cursor, line_no)?;
            records[det_idx]
                .subtables
                .entry((*name).to_string())
                .or_default();
            Ok(Cursor::Subtable(det_idx, (*name).to_string()))
        }
        _ => Err(ParseError {
            line: line_no,
            message: format!("unsupported table header `[{inner}]`"),
        }),
    }
}

fn assign_into_cursor(
    records: &mut [DetectorRecord],
    cursor: &Cursor,
    key: String,
    value: Value,
    line_no: usize,
) -> Result<(), ParseError> {
    match cursor {
        Cursor::TopLevel => Err(ParseError {
            line: line_no,
            message: "assignment outside any [[detector]] section".to_string(),
        }),
        Cursor::Detector(idx) => {
            records[*idx].fields.insert(key, value);
            Ok(())
        }
        Cursor::Subtable(idx, name) => {
            let table = records[*idx].subtables.entry(name.clone()).or_default();
            table.insert(key, value);
            Ok(())
        }
        Cursor::ArraySubtable(idx, name, sub_idx) => {
            let arr = records[*idx]
                .array_subtables
                .entry(name.clone())
                .or_default();
            while arr.len() <= *sub_idx {
                arr.push(BTreeMap::new());
            }
            arr[*sub_idx].insert(key, value);
            Ok(())
        }
    }
}

fn current_detector_index(cursor: &Cursor, line: usize) -> Result<usize, ParseError> {
    match cursor {
        Cursor::TopLevel => Err(ParseError {
            line,
            message: "subtable header without preceding [[detector]]".to_string(),
        }),
        Cursor::Detector(i) | Cursor::Subtable(i, _) | Cursor::ArraySubtable(i, _, _) => Ok(*i),
    }
}

enum Cursor {
    TopLevel,
    Detector(usize),
    Subtable(usize, String),
    ArraySubtable(usize, String, usize),
}

fn strip_comment(line: &str) -> &str {
    // `#` outside a quoted string starts a comment. We scan the line
    // once, tracking whether we're inside a `"..."` literal, and cut
    // at the first un-quoted `#`.
    let bytes = line.as_bytes();
    let mut in_quotes = false;
    let mut escape = false;
    for (i, &b) in bytes.iter().enumerate() {
        if escape {
            escape = false;
            continue;
        }
        match b {
            b'\\' if in_quotes => escape = true,
            b'"' => in_quotes = !in_quotes,
            b'#' if !in_quotes => return &line[..i],
            _ => {}
        }
    }
    line
}

fn section_header<'a>(line: &'a str, open: &str, close: &str) -> Option<&'a str> {
    let line = line.trim();
    if !line.starts_with(open) || !line.ends_with(close) {
        return None;
    }
    let body = &line[open.len()..line.len() - close.len()];
    // Reject `[[detector]` (mismatched brackets); `section_header` is
    // called with the right opener/closer pair already.
    if body.contains('[') || body.contains(']') {
        return None;
    }
    Some(body.trim())
}

fn parse_assignment(line: &str, line_no: usize) -> Result<(String, Value), ParseError> {
    let eq = line.find('=').ok_or_else(|| ParseError {
        line: line_no,
        message: format!("expected `key = value`, got `{line}`"),
    })?;
    let key = line[..eq].trim().to_string();
    if key.is_empty() {
        return Err(ParseError {
            line: line_no,
            message: "empty key on left of `=`".to_string(),
        });
    }
    let rest = line[eq + 1..].trim();
    let value = parse_value(rest, line_no)?;
    Ok((key, value))
}

fn parse_value(s: &str, line_no: usize) -> Result<Value, ParseError> {
    let s = s.trim();
    if s.is_empty() {
        return Err(ParseError {
            line: line_no,
            message: "empty value".to_string(),
        });
    }
    if s == "true" {
        return Ok(Value::Bool(true));
    }
    if s == "false" {
        return Ok(Value::Bool(false));
    }
    if s.starts_with('"') {
        let (text, rest) = parse_string(s, line_no)?;
        if !rest.trim().is_empty() {
            return Err(ParseError {
                line: line_no,
                message: format!("trailing characters after string: `{}`", rest.trim()),
            });
        }
        return Ok(Value::String(text));
    }
    if s.starts_with('[') {
        return parse_array(s, line_no);
    }
    // Integer
    parse_int(s, line_no).map(Value::Int)
}

fn parse_int(s: &str, line_no: usize) -> Result<i64, ParseError> {
    s.trim().parse::<i64>().map_err(|err| ParseError {
        line: line_no,
        message: format!("invalid integer `{s}`: {err}"),
    })
}

/// Parse a TOML basic string starting at `s[0] == '"'`.
/// Returns the decoded text and the rest of the slice after the
/// closing quote.
fn parse_string(s: &str, line_no: usize) -> Result<(String, &str), ParseError> {
    if !s.starts_with('"') {
        return Err(ParseError {
            line: line_no,
            message: "expected `\"` to start string".to_string(),
        });
    }
    let bytes = s.as_bytes();
    let mut out = String::new();
    let mut i = 1usize;
    while i < bytes.len() {
        match bytes[i] {
            b'\\' => {
                i += 1;
                if i >= bytes.len() {
                    return Err(ParseError {
                        line: line_no,
                        message: "trailing backslash in string".to_string(),
                    });
                }
                match bytes[i] {
                    b'"' => out.push('"'),
                    b'\\' => out.push('\\'),
                    b'n' => out.push('\n'),
                    b't' => out.push('\t'),
                    b'r' => out.push('\r'),
                    other => {
                        return Err(ParseError {
                            line: line_no,
                            message: format!("unknown escape `\\{}`", other as char),
                        });
                    }
                }
                i += 1;
            }
            b'"' => {
                return Ok((out, &s[i + 1..]));
            }
            other => {
                out.push(other as char);
                i += 1;
            }
        }
    }
    Err(ParseError {
        line: line_no,
        message: "unterminated string literal".to_string(),
    })
}

fn parse_array(s: &str, line_no: usize) -> Result<Value, ParseError> {
    let s = s.trim();
    if !s.starts_with('[') || !s.ends_with(']') {
        return Err(ParseError {
            line: line_no,
            message: format!("expected `[...]`, got `{s}`"),
        });
    }
    let body = s[1..s.len() - 1].trim();
    if body.is_empty() {
        return Ok(Value::Array(Vec::new()));
    }
    let mut elements: Vec<Value> = Vec::new();
    let mut rest = body;
    loop {
        let trimmed = rest.trim_start();
        if trimmed.is_empty() {
            break;
        }
        // Parse one element
        let (element, after) = parse_array_element(trimmed, line_no)?;
        elements.push(element);
        let after = after.trim_start();
        if after.is_empty() {
            break;
        }
        if !after.starts_with(',') {
            return Err(ParseError {
                line: line_no,
                message: format!("expected `,` in array, got `{after}`"),
            });
        }
        rest = &after[1..];
    }
    Ok(Value::Array(elements))
}

fn parse_array_element(s: &str, line_no: usize) -> Result<(Value, &str), ParseError> {
    let s = s.trim_start();
    if s.starts_with('"') {
        let (text, rest) = parse_string(s, line_no)?;
        return Ok((Value::String(text), rest));
    }
    if s.starts_with("true") && peek_terminator(&s[4..]) {
        return Ok((Value::Bool(true), &s[4..]));
    }
    if s.starts_with("false") && peek_terminator(&s[5..]) {
        return Ok((Value::Bool(false), &s[5..]));
    }
    // Integer: consume while in `[0-9-]`.
    let mut end = 0;
    let bytes = s.as_bytes();
    if !bytes.is_empty() && (bytes[0] == b'-' || bytes[0].is_ascii_digit()) {
        end += 1;
        while end < bytes.len() && bytes[end].is_ascii_digit() {
            end += 1;
        }
    }
    if end == 0 {
        return Err(ParseError {
            line: line_no,
            message: format!("expected value in array, got `{}`", &s[..s.len().min(20)]),
        });
    }
    let int = parse_int(&s[..end], line_no)?;
    Ok((Value::Int(int), &s[end..]))
}

fn peek_terminator(s: &str) -> bool {
    let s = s.trim_start();
    s.is_empty() || s.starts_with(',') || s.starts_with(']')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_minimal_record() {
        let src = "
[[detector]]
canonical_id = 1
display_name = \"Foo\"
is_origin = true

[detector.parameter_bounds]
axis_count = 2
description = \"two axes\"

[[detector.source_refs]]
citation_key = \"foo1924\"
year = 1924
";
        let recs = parse_detectors(src).unwrap();
        assert_eq!(recs.len(), 1);
        assert_eq!(recs[0].fields["canonical_id"], Value::Int(1));
        assert_eq!(recs[0].fields["display_name"], Value::String("Foo".into()));
        assert_eq!(recs[0].fields["is_origin"], Value::Bool(true));
        assert_eq!(
            recs[0].subtables["parameter_bounds"]["axis_count"],
            Value::Int(2)
        );
        assert_eq!(
            recs[0].array_subtables["source_refs"][0]["citation_key"],
            Value::String("foo1924".into())
        );
    }

    #[test]
    fn parses_string_arrays_and_empty_arrays() {
        let src = "
[[detector]]
canonical_id = 1
aliases = [\"a\", \"b\", \"c\"]
empty = []
ints = [10, 20, 30]
";
        let recs = parse_detectors(src).unwrap();
        let Value::Array(arr) = &recs[0].fields["aliases"] else {
            panic!()
        };
        assert_eq!(arr.len(), 3);
        let Value::Array(empty) = &recs[0].fields["empty"] else {
            panic!()
        };
        assert!(empty.is_empty());
        let Value::Array(ints) = &recs[0].fields["ints"] else {
            panic!()
        };
        assert_eq!(ints[1], Value::Int(20));
    }

    #[test]
    fn comments_and_blank_lines_ignored() {
        let src = "
# leading comment
[[detector]]
# inside the record
canonical_id = 1   # trailing comment

display_name = \"After blank\"
";
        let recs = parse_detectors(src).unwrap();
        assert_eq!(recs[0].fields["canonical_id"], Value::Int(1));
        assert_eq!(
            recs[0].fields["display_name"],
            Value::String("After blank".into())
        );
    }

    #[test]
    fn string_escapes_round_trip() {
        let src = "
[[detector]]
quoted = \"a \\\"b\\\" c\"
newline = \"x\\ny\"
backslash = \"x\\\\y\"
";
        let recs = parse_detectors(src).unwrap();
        assert_eq!(recs[0].fields["quoted"], Value::String("a \"b\" c".into()));
        assert_eq!(recs[0].fields["newline"], Value::String("x\ny".into()));
        assert_eq!(recs[0].fields["backslash"], Value::String("x\\y".into()));
    }

    #[test]
    fn rejects_assignment_outside_record() {
        let src = "key = 1\n";
        let err = parse_detectors(src).unwrap_err();
        assert_eq!(err.line, 1);
        assert!(err.message.contains("outside any"));
    }

    #[test]
    fn rejects_unterminated_string() {
        let src = "
[[detector]]
oops = \"missing close
";
        let err = parse_detectors(src).unwrap_err();
        assert!(err.message.contains("unterminated"));
    }
}
