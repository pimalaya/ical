//! # Escape (write codec)
//!
//! Apply the RFC 5545 3.3.11 value escapes when serializing.
//!
//! This is the write half of the escaping codec; its inverse is
//! [`unescape`](crate::tree::codec::unescape), and the version-specific rules
//! are selected by the [`Escaper`]. The structural encoders in
//! [`encode`](crate::tree::codec::encode) run every value leaf through here.
//!
//! No mode ever writes a byte that would end the line, whatever a caller put
//! in the value.
//!
//! Where a version has no escape for one, as vCalendar 1.0 has none for a
//! newline, the escape that keeps the calendar readable is written and the
//! round trip through [`unescape`](crate::tree::codec::unescape) is not exact.
//!
//! A parameter value is a different alphabet and has its own writer,
//! `escape_param`, applying the RFC 6868 caret encoding rather than any
//! backslash one.

use alloc::{borrow::Cow, string::String, vec::Vec};

use crate::tree::codec::mode::Escaper;

/// Apply the value escapes by the calendar's escaping mode, over raw bytes.
///
/// RFC 5545 3.3.11 for the modern rules, `;` and a newline for vCalendar 1.0.
/// Borrows when nothing needs escaping; non-UTF-8 content passes through
/// verbatim.
pub(crate) fn escape_with(bytes: &[u8], escaper: Escaper) -> Cow<'_, [u8]> {
    if escaped_at(bytes, escaper).is_none() {
        return Cow::Borrowed(bytes);
    }

    let mut out = Vec::with_capacity(bytes.len() + 2);
    escape_into(bytes, escaper, &mut out);
    Cow::Owned(out)
}

/// Append `bytes` to `out` with the value escapes of the mode applied, as
/// [`escape_with`] returns them.
pub(crate) fn escape_into(bytes: &[u8], escaper: Escaper, out: &mut Vec<u8>) {
    let mut rest = bytes;

    while let Some(at) = escaped_at(rest, escaper) {
        out.extend_from_slice(&rest[..at]);
        out.extend_from_slice(escape_of(rest[at]));
        rest = &rest[at + 1..];
    }

    out.extend_from_slice(rest);
}

/// The index of the first byte a value escape rewrites, by escaping mode:
/// `\`, `,`, `;` and a newline in RFC 5545 3.3.11, `;` and a newline in
/// vCalendar 1.0.
///
/// Versit has no newline escape, so a newline written into a 1.0 value goes
/// out as `\n` and reads back as those two characters: that is the closest 1.0
/// can carry it, left raw it would end the line and the calendar would not
/// parse.
fn escaped_at(bytes: &[u8], escaper: Escaper) -> Option<usize> {
    match escaper {
        Escaper::Modern => position(bytes, |b| {
            (b == b'\\') | (b == b',') | (b == b';') | (b == b'\n')
        }),
        Escaper::V1_0 => memchr::memchr2(b';', b'\n', bytes),
    }
}

/// The escape written for a byte [`escaped_at`] finds.
fn escape_of(byte: u8) -> &'static [u8] {
    match byte {
        b'\\' => br"\\",
        b',' => br"\,",
        b';' => br"\;",
        _ => br"\n",
    }
}

/// The index of the first byte `hit` matches.
///
/// Sixteen bytes at a time, each block's hits gathered into a bit mask with no
/// early exit, which compiles to vector compares and a mask move, so a long
/// value passes at several bytes a cycle. `hit` must join its tests with `|`:
/// a `matches!` compiles to a branch per byte.
fn position(bytes: &[u8], hit: impl Fn(u8) -> bool) -> Option<usize> {
    let (blocks, tail) = bytes.as_chunks::<16>();

    for (i, block) in blocks.iter().enumerate() {
        let mut mask = 0u16;

        for (j, &b) in block.iter().enumerate() {
            mask |= u16::from(hit(b)) << j;
        }

        if mask != 0 {
            return Some(i * 16 + mask.trailing_zeros() as usize);
        }
    }

    tail.iter()
        .position(|&b| hit(b))
        .map(|j| blocks.len() * 16 + j)
}

/// Apply the RFC 6868 3.1 parameter value encoding (a newline as `^n`, a caret
/// as `^^` and a double quote as `^'`), then wrap the result in the RFC 5545
/// 3.1 delimiters when it needs them. The inverse of
/// [`unescape_param`](crate::tree::codec::unescape::unescape_param).
///
/// A version predating RFC 6868 is written with no parameter encoding at all,
/// and one predating the `quoted-string` production with no quoting: see
/// [`Escaper::has_param_encoding`] and [`Escaper::has_param_quoting`].
pub(crate) fn escape_param(value: &str, escaper: Escaper) -> Cow<'_, str> {
    let value = match escaper.has_param_encoding() {
        true => escape_carets(value),
        false => Cow::Borrowed(value),
    };

    if !escaper.has_param_quoting() || !value.bytes().any(|b| matches!(b, b',' | b';' | b':')) {
        return value;
    }

    // NOTE: a double quote never reaches here, the caret encoding having
    // spelled it `^'`, so the pair written below is unambiguously the
    // production's own.
    let mut out = String::with_capacity(value.len() + 2);
    out.push('"');
    out.push_str(&value);
    out.push('"');
    Cow::Owned(out)
}

/// Apply the RFC 6868 caret encoding over every character of `value`.
fn escape_carets(value: &str) -> Cow<'_, str> {
    if !value.bytes().any(|b| matches!(b, b'\n' | b'^' | b'"')) {
        return Cow::Borrowed(value);
    }

    let mut out = String::with_capacity(value.len());

    for c in value.chars() {
        match c {
            '\n' => out.push_str("^n"),
            '^' => out.push_str("^^"),
            '"' => out.push_str("^'"),
            other => out.push(other),
        }
    }

    Cow::Owned(out)
}

#[cfg(test)]
mod tests {
    use alloc::borrow::Cow;

    use crate::tree::codec::{
        escape::{escape_param, escape_with},
        mode::Escaper,
    };

    #[test]
    fn escapes_separators_and_newlines_and_borrows_when_clean() {
        assert_eq!(
            escape_with(b"a,b;c\nd", Escaper::Modern).as_ref(),
            br"a\,b\;c\nd".as_slice(),
        );
        assert!(matches!(
            escape_with(b"plain", Escaper::Modern),
            Cow::Borrowed(b"plain")
        ));
        // NOTE: vCalendar 1.0 escapes `;`, and a newline as `\n` for want of
        // an escape of its own, which would otherwise end the line.
        assert_eq!(
            escape_with(b"a,b;c\nd", Escaper::V1_0).as_ref(),
            br"a,b\;c\nd".as_slice(),
        );
    }

    #[test]
    fn encodes_the_rfc_6868_parameter_sequences_and_borrows_when_clean() {
        assert_eq!(escape_param("a\nb^c\"d", Escaper::Modern), "a^nb^^c^'d");
        assert!(matches!(
            escape_param("plain", Escaper::Modern),
            Cow::Borrowed("plain")
        ));
        // NOTE: RFC 6868 section 3.2 forbids backslash escaping, so a path
        // keeps its backslash; its colon is what the quotes are for.
        assert_eq!(escape_param(r"C:\temp", Escaper::Modern), r#""C:\temp""#);
    }

    /// RFC 5545 section 3.1 keeps `,`, `;` and `:` out of a bare `paramtext`,
    /// so a value carrying one is wrapped and a value carrying none is not:
    /// the quotes are the grammar's, not the value's.
    #[test]
    fn quotes_a_parameter_value_only_where_a_delimiter_needs_it() {
        assert_eq!(
            escape_param("cid:part1.0001@example.org", Escaper::Modern),
            "\"cid:part1.0001@example.org\"",
        );
        assert_eq!(
            escape_param("America/New_York", Escaper::Modern),
            "America/New_York",
        );
        assert!(matches!(
            escape_param("CHAIR", Escaper::Modern),
            Cow::Borrowed("CHAIR")
        ));
    }

    /// A double quote is content, so it goes out RFC 6868 encoded rather than
    /// as a delimiter, and the pair the comma calls for is added around it.
    #[test]
    fn encodes_a_double_quote_rather_than_reading_it_as_a_delimiter() {
        assert_eq!(
            escape_param("say \"hi\", then go", Escaper::Modern),
            "\"say ^'hi^', then go\"",
        );
    }

    /// vCalendar 1.0 has no `quoted-string`, so nothing is wrapped: a
    /// delimiter goes out bare, as every 1.0 writer puts it.
    #[test]
    fn never_quotes_a_vcalendar_1_0_parameter_value() {
        assert!(matches!(
            escape_param("a,b", Escaper::V1_0),
            Cow::Borrowed("a,b")
        ));
    }

    #[test]
    fn writes_a_vcalendar_1_0_parameter_unencoded() {
        // NOTE: RFC 6868 updates RFC 5545 alone, so a 1.0 caret goes out as
        // itself and a 1.0 reader would not resolve `^^` anyway.
        assert!(matches!(
            escape_param("a^b", Escaper::V1_0),
            Cow::Borrowed("a^b")
        ));
    }
}
