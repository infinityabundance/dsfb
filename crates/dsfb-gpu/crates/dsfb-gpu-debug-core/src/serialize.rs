//! Canonical-JSON serialization and parsing for fixtures and case files.
//!
//! Why this exists: every artifact whose hash is chained into the verdict
//! must be representable in a byte-exact canonical form. Pulling in `serde`
//! or any other framework would break the zero-dependency posture of the
//! core crate and, more importantly, would push the byte form into an
//! upstream's hands — patch-level changes there would silently break the
//! prior-art replay claim.
//!
//! The format we write and read is a strict subset of JSON suitable for
//! audit:
//!
//! * No whitespace anywhere (no spaces, tabs, or newlines).
//! * Object keys are emitted in lexicographic byte order.
//! * Integers as plain ASCII digits, no leading zeros, no exponents.
//! * Strings restricted to printable ASCII; the only escape produced is
//!   `\"`. The fixtures and case files never contain non-ASCII payloads,
//!   so a richer escape set would be dead weight.
//! * Booleans as `true` / `false`.
//!
//! The writers are typed end-to-end (one function per domain type) rather
//! than building a generic `Value` enum and serializing it. Same for the
//! parsers. This keeps every byte that ends up in the hash chain visible
//! at its emission site.
//!
//! Only available when the `std` feature is on; the readers and writers
//! need `Vec<u8>` to accumulate output.

#![cfg(feature = "std")]

use std::vec::Vec;

use crate::event::TraceEvent;

/// Errors that the canonical readers can surface. Each variant carries the
/// byte offset at which the problem was detected so audit logs can pinpoint
/// where a malformed input deviated from the contract.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ParseError {
    /// The input ended before the expected token.
    Truncated {
        /// Byte offset at which the parser ran out of input.
        at: usize,
    },
    /// A byte that didn't match what the canonical grammar required here.
    Unexpected {
        /// Byte offset of the offending byte.
        at: usize,
        /// The byte that was actually present.
        found: u8,
        /// Short label describing what the grammar expected at this point.
        wanted: &'static str,
    },
    /// A field appeared more than once, or out of canonical order.
    OutOfOrderField {
        /// Byte offset at which the key opened.
        at: usize,
        /// Name of the field that was expected next.
        name: &'static str,
    },
    /// An integer value did not fit in the declared width.
    Overflow {
        /// Byte offset at which the integer started.
        at: usize,
        /// Name of the field whose value overflowed.
        name: &'static str,
    },
    /// A required field was missing.
    MissingField {
        /// Name of the missing field.
        name: &'static str,
    },
    /// A required version string didn't match the contract.
    VersionMismatch {
        /// The version string the parser expected.
        wanted: &'static str,
    },
}

/// Convert an arbitrary `u64` to canonical ASCII digits, appending to the
/// provided buffer. Hand-rolled because `Vec::extend_from_slice` plus
/// `format!("{n}")` would allocate a temporary; this writes straight into
/// the destination.
pub fn write_u64(buf: &mut Vec<u8>, mut n: u64) {
    if n == 0 {
        buf.push(b'0');
        return;
    }
    // 20 digits fits any u64. Build in a fixed-size scratch from the low
    // end up, then copy the produced digits in forward order into the
    // destination so the canonical bytes stay big-endian numeric order.
    let mut scratch = [0u8; 20];
    let mut i = scratch.len();
    while n > 0 {
        i -= 1;
        scratch[i] = b'0' + (n % 10) as u8;
        n /= 10;
    }
    buf.extend_from_slice(&scratch[i..]);
}

/// Write a `bool` as canonical `true` or `false`.
pub fn write_bool(buf: &mut Vec<u8>, b: bool) {
    if b {
        buf.extend_from_slice(b"true");
    } else {
        buf.extend_from_slice(b"false");
    }
}

/// Write a string literal. Restricted to printable ASCII. Panics in debug
/// builds if the input contains control bytes or non-ASCII — the fixtures
/// and case files never emit such strings by design, and a silent failure
/// here would corrupt the hash chain.
pub fn write_str(buf: &mut Vec<u8>, s: &str) {
    buf.push(b'"');
    for &byte in s.as_bytes() {
        debug_assert!(
            (0x20..0x7f).contains(&byte),
            "non-ASCII byte in canonical string: {byte:#x}"
        );
        if byte == b'"' || byte == b'\\' {
            buf.push(b'\\');
        }
        buf.push(byte);
    }
    buf.push(b'"');
}

/// Append a literal `"key":` prefix. Convenience for object writers.
pub fn write_key(buf: &mut Vec<u8>, key: &str) {
    write_str(buf, key);
    buf.push(b':');
}

/// Serialize a `TraceEvent` to canonical bytes, appending to `buf`.
///
/// The field order in the output is lexicographic by key name, not the
/// declaration order of the struct. This is what makes the bytes
/// canonical: any consumer that sorts keys before hashing will reproduce
/// the same hash.
pub fn write_event(buf: &mut Vec<u8>, event: &TraceEvent) {
    buf.push(b'{');

    // Keys in lexicographic order: entity_id, error_code, event_kind,
    // flags, latency_us, parent_span_id, route_id, span_id, status_code,
    // ts_ns. Ten fields total.

    write_key(buf, "entity_id");
    write_u64(buf, u64::from(event.entity_id));
    buf.push(b',');

    write_key(buf, "error_code");
    write_u64(buf, u64::from(event.error_code));
    buf.push(b',');

    write_key(buf, "event_kind");
    write_u64(buf, u64::from(event.event_kind));
    buf.push(b',');

    write_key(buf, "flags");
    write_u64(buf, u64::from(event.flags));
    buf.push(b',');

    write_key(buf, "latency_us");
    write_u64(buf, u64::from(event.latency_us));
    buf.push(b',');

    write_key(buf, "parent_span_id");
    write_u64(buf, event.parent_span_id);
    buf.push(b',');

    write_key(buf, "route_id");
    write_u64(buf, u64::from(event.route_id));
    buf.push(b',');

    write_key(buf, "span_id");
    write_u64(buf, event.span_id);
    buf.push(b',');

    write_key(buf, "status_code");
    write_u64(buf, u64::from(event.status_code));
    buf.push(b',');

    write_key(buf, "ts_ns");
    write_u64(buf, event.ts_ns);

    buf.push(b'}');
}

/// Canonical fixture wrapper format:
///
/// ```text
/// {"events":[<event>,<event>,...],"n_events":<N>,"version":"<v>"}
/// ```
///
/// `version` is recorded so future shape changes can be detected at parse
/// time without ambiguity.
pub const FIXTURE_VERSION: &str = "DSFB-GPU-DEBUG-FIXTURE-0.1";

/// Serialize a slice of `TraceEvent`s as a complete canonical fixture file.
#[must_use]
pub fn write_fixture(events: &[TraceEvent]) -> Vec<u8> {
    let mut buf: Vec<u8> = Vec::with_capacity(events.len() * 192);
    buf.push(b'{');
    write_key(&mut buf, "events");
    buf.push(b'[');
    for (i, event) in events.iter().enumerate() {
        if i > 0 {
            buf.push(b',');
        }
        write_event(&mut buf, event);
    }
    buf.push(b']');
    buf.push(b',');
    write_key(&mut buf, "n_events");
    write_u64(&mut buf, events.len() as u64);
    buf.push(b',');
    write_key(&mut buf, "version");
    write_str(&mut buf, FIXTURE_VERSION);
    buf.push(b'}');
    buf
}

/// A bytes-and-cursor parser for canonical JSON fixtures. Intentionally
/// dumb — it only understands what `write_*` produces and will refuse
/// anything else.
struct Parser<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> Parser<'a> {
    const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, pos: 0 }
    }

    fn peek(&self) -> Result<u8, ParseError> {
        self.bytes
            .get(self.pos)
            .copied()
            .ok_or(ParseError::Truncated { at: self.pos })
    }

    fn expect(&mut self, wanted: u8, label: &'static str) -> Result<(), ParseError> {
        let found = self.peek()?;
        if found != wanted {
            return Err(ParseError::Unexpected {
                at: self.pos,
                found,
                wanted: label,
            });
        }
        self.pos += 1;
        Ok(())
    }

    /// Read a canonical unsigned integer (decimal ASCII, no leading zeros
    /// except for the value 0 itself). Returns the value plus the byte
    /// offset at which the integer started, for diagnostics.
    fn read_u64(&mut self, label: &'static str) -> Result<u64, ParseError> {
        let start = self.pos;
        let mut value: u64 = 0;
        let mut digits = 0;
        while let Some(byte @ b'0'..=b'9') = self.bytes.get(self.pos).copied() {
            value = value
                .checked_mul(10)
                .and_then(|v| v.checked_add(u64::from(byte - b'0')))
                .ok_or(ParseError::Overflow {
                    at: start,
                    name: label,
                })?;
            self.pos += 1;
            digits += 1;
        }
        if digits == 0 {
            return Err(ParseError::Unexpected {
                at: start,
                found: self.peek().unwrap_or(b'\0'),
                wanted: "digit",
            });
        }
        // Reject canonical-violating leading zero (e.g. "01") for any value
        // other than the single literal "0".
        if digits > 1 && self.bytes[start] == b'0' {
            return Err(ParseError::Unexpected {
                at: start,
                found: b'0',
                wanted: "no-leading-zero-integer",
            });
        }
        Ok(value)
    }

    fn read_u32(&mut self, label: &'static str) -> Result<u32, ParseError> {
        let pos_before = self.pos;
        let v = self.read_u64(label)?;
        if v > u64::from(u32::MAX) {
            return Err(ParseError::Overflow {
                at: pos_before,
                name: label,
            });
        }
        Ok(v as u32)
    }

    fn read_u16(&mut self, label: &'static str) -> Result<u16, ParseError> {
        let pos_before = self.pos;
        let v = self.read_u64(label)?;
        if v > u64::from(u16::MAX) {
            return Err(ParseError::Overflow {
                at: pos_before,
                name: label,
            });
        }
        Ok(v as u16)
    }

    /// Match a literal key `"name":`. Used by the object readers to walk
    /// keys in canonical lexicographic order — out-of-order keys are an
    /// error.
    fn expect_key(&mut self, name: &'static str) -> Result<(), ParseError> {
        let start = self.pos;
        self.expect(b'"', "string-start")?;
        for byte in name.as_bytes() {
            let actual = self.peek()?;
            if actual != *byte {
                return Err(ParseError::OutOfOrderField { at: start, name });
            }
            self.pos += 1;
        }
        self.expect(b'"', "string-end")?;
        self.expect(b':', "colon")?;
        Ok(())
    }

    /// Read a quoted string and assert it equals `expected`. Used for
    /// version pins and other canonical-literal fields.
    fn expect_str_literal(&mut self, expected: &'static str) -> Result<(), ParseError> {
        self.expect(b'"', "string-start")?;
        for byte in expected.as_bytes() {
            let actual = self.peek()?;
            if actual != *byte {
                return Err(ParseError::VersionMismatch { wanted: expected });
            }
            self.pos += 1;
        }
        self.expect(b'"', "string-end")?;
        Ok(())
    }
}

/// Read a `TraceEvent` from canonical bytes at the parser's current
/// position. Field order is enforced strictly — out-of-order keys raise
/// `OutOfOrderField`.
fn read_event(p: &mut Parser<'_>) -> Result<TraceEvent, ParseError> {
    p.expect(b'{', "object-start")?;

    p.expect_key("entity_id")?;
    let entity_id = p.read_u32("entity_id")?;
    p.expect(b',', "field-sep")?;

    p.expect_key("error_code")?;
    let error_code = p.read_u16("error_code")?;
    p.expect(b',', "field-sep")?;

    p.expect_key("event_kind")?;
    let event_kind = p.read_u16("event_kind")?;
    p.expect(b',', "field-sep")?;

    p.expect_key("flags")?;
    let flags = p.read_u16("flags")?;
    p.expect(b',', "field-sep")?;

    p.expect_key("latency_us")?;
    let latency_us = p.read_u32("latency_us")?;
    p.expect(b',', "field-sep")?;

    p.expect_key("parent_span_id")?;
    let parent_span_id = p.read_u64("parent_span_id")?;
    p.expect(b',', "field-sep")?;

    p.expect_key("route_id")?;
    let route_id = p.read_u32("route_id")?;
    p.expect(b',', "field-sep")?;

    p.expect_key("span_id")?;
    let span_id = p.read_u64("span_id")?;
    p.expect(b',', "field-sep")?;

    p.expect_key("status_code")?;
    let status_code = p.read_u16("status_code")?;
    p.expect(b',', "field-sep")?;

    p.expect_key("ts_ns")?;
    let ts_ns = p.read_u64("ts_ns")?;

    p.expect(b'}', "object-end")?;

    Ok(TraceEvent {
        ts_ns,
        entity_id,
        route_id,
        span_id,
        parent_span_id,
        latency_us,
        status_code,
        error_code,
        event_kind,
        flags,
    })
}

/// Parse a canonical fixture file and return its event list. Validates the
/// version string and the recorded `n_events` against the actual event
/// count.
///
/// # Errors
///
/// Returns a [`ParseError`] when the bytes deviate from canonical form,
/// including: unexpected bytes, out-of-order keys, overflow, missing
/// fields, and version mismatches.
pub fn read_fixture(bytes: &[u8]) -> Result<Vec<TraceEvent>, ParseError> {
    let mut p = Parser::new(bytes);
    p.expect(b'{', "object-start")?;
    p.expect_key("events")?;
    p.expect(b'[', "array-start")?;

    let mut events: Vec<TraceEvent> = Vec::new();
    // Empty array shortcut so the `,`/`]` look-ahead below is simpler.
    if p.peek()? != b']' {
        loop {
            events.push(read_event(&mut p)?);
            match p.peek()? {
                b',' => {
                    p.pos += 1;
                }
                b']' => break,
                other => {
                    return Err(ParseError::Unexpected {
                        at: p.pos,
                        found: other,
                        wanted: "array-sep-or-end",
                    })
                }
            }
        }
    }
    p.expect(b']', "array-end")?;
    p.expect(b',', "field-sep")?;

    p.expect_key("n_events")?;
    let recorded = p.read_u64("n_events")?;
    if recorded != events.len() as u64 {
        return Err(ParseError::Overflow {
            at: p.pos,
            name: "n_events",
        });
    }
    p.expect(b',', "field-sep")?;

    p.expect_key("version")?;
    p.expect_str_literal(FIXTURE_VERSION)?;

    p.expect(b'}', "object-end")?;

    Ok(events)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_u64_canonical_form() {
        let cases: [(u64, &[u8]); 6] = [
            (0, b"0"),
            (1, b"1"),
            (10, b"10"),
            (255, b"255"),
            (1_234_567_890, b"1234567890"),
            (u64::MAX, b"18446744073709551615"),
        ];
        for (n, expected) in cases {
            let mut buf = Vec::new();
            write_u64(&mut buf, n);
            assert_eq!(&buf[..], expected, "for n={n}");
        }
    }

    #[test]
    fn event_round_trip_is_byte_stable() {
        let event = TraceEvent::new(
            1_234_567_890,
            5,
            17,
            0xDEAD_BEEF_CAFE_F00D,
            0xBABE_FACE_1234_5678,
            42_000,
            200,
            0,
            1,
            0,
        );
        let mut buf = Vec::new();
        write_event(&mut buf, &event);
        let s = std::str::from_utf8(&buf).unwrap();
        // Field order must be lexicographic. Each key/value pair is verified
        // individually so the test does not depend on writing out the
        // exact decimal expansion of two u64 span identifiers.
        let expected_prefix = format!(
            r#"{{"entity_id":5,"error_code":0,"event_kind":1,"flags":0,"latency_us":42000,"parent_span_id":{psid},"route_id":17,"span_id":{sid},"status_code":200,"ts_ns":1234567890}}"#,
            psid = 0xBABE_FACE_1234_5678_u64,
            sid = 0xDEAD_BEEF_CAFE_F00D_u64,
        );
        assert_eq!(s, expected_prefix);
    }

    #[test]
    fn fixture_round_trip_preserves_events() {
        let events = vec![
            TraceEvent::new(0, 0, 0, 0, 0, 1000, 200, 0, 0, 0),
            TraceEvent::new(1_000_000, 1, 2, 3, 4, 2000, 200, 0, 0, 0),
            TraceEvent::new(2_000_000, 2, 3, 4, 5, 3000, 500, 500, 0, 0),
        ];
        let bytes = write_fixture(&events);
        let parsed = read_fixture(&bytes).expect("round-trip parses");
        assert_eq!(parsed, events);
    }

    #[test]
    fn fixture_round_trip_byte_identical_after_reserialize() {
        let events = vec![
            TraceEvent::new(0, 0, 0, 0, 0, 1000, 200, 0, 0, 0),
            TraceEvent::new(1_000_000, 15, 63, u64::MAX, 0, 32_000_000, 503, 503, 2, 1),
        ];
        let bytes_a = write_fixture(&events);
        let parsed = read_fixture(&bytes_a).expect("round-trip parses");
        let bytes_b = write_fixture(&parsed);
        assert_eq!(
            bytes_a, bytes_b,
            "canonical bytes are byte-stable across re-serialize"
        );
    }

    #[test]
    fn read_fixture_rejects_wrong_version() {
        // Build a fixture with a bogus version string by hand.
        let mut buf = Vec::new();
        buf.extend_from_slice(br#"{"events":[],"n_events":0,"version":"BOGUS"}"#);
        let err = read_fixture(&buf).unwrap_err();
        assert!(matches!(err, ParseError::VersionMismatch { .. }));
    }

    #[test]
    fn read_fixture_rejects_out_of_order_keys() {
        // Build a fixture where the event has fields out of canonical order.
        // We swap entity_id and error_code on purpose.
        let mut buf = Vec::new();
        buf.extend_from_slice(
            br#"{"events":[{"error_code":0,"entity_id":0,"event_kind":0,"flags":0,"latency_us":0,"parent_span_id":0,"route_id":0,"span_id":0,"status_code":0,"ts_ns":0}],"n_events":1,"version":"DSFB-GPU-DEBUG-FIXTURE-0.1"}"#,
        );
        let err = read_fixture(&buf).unwrap_err();
        assert!(matches!(
            err,
            ParseError::OutOfOrderField {
                name: "entity_id",
                ..
            }
        ));
    }

    #[test]
    fn read_fixture_rejects_count_mismatch() {
        // Empty event array but n_events says 5.
        let mut buf = Vec::new();
        buf.extend_from_slice(
            br#"{"events":[],"n_events":5,"version":"DSFB-GPU-DEBUG-FIXTURE-0.1"}"#,
        );
        let err = read_fixture(&buf).unwrap_err();
        assert!(matches!(
            err,
            ParseError::Overflow {
                name: "n_events",
                ..
            }
        ));
    }

    #[test]
    fn read_fixture_rejects_leading_zero_integers() {
        // "01" is not canonical even though it parses to 1 elsewhere.
        let mut buf = Vec::new();
        buf.extend_from_slice(
            br#"{"events":[{"entity_id":01,"error_code":0,"event_kind":0,"flags":0,"latency_us":0,"parent_span_id":0,"route_id":0,"span_id":0,"status_code":0,"ts_ns":0}],"n_events":1,"version":"DSFB-GPU-DEBUG-FIXTURE-0.1"}"#,
        );
        let err = read_fixture(&buf).unwrap_err();
        assert!(matches!(
            err,
            ParseError::Unexpected {
                wanted: "no-leading-zero-integer",
                ..
            }
        ));
    }
}
