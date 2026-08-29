//! # Escape (write codec)
//!
//! Apply the RFC 5545 3.3.11 value escapes when serializing. The write half of
//! the escaping codec; its inverse is
//! [`unescape`](crate::tree::codec::unescape), and the version-specific rules
//! are selected by the [`Escaper`]. The structural encoders in
//! [`encode`](crate::tree::codec::encode) run every value leaf through here.
//!
//! No mode ever writes a byte that would end the line, whatever a caller put in
//! the value. Where a version has no escape for one, as vCalendar 1.0 has none
//! for a newline, the escape that keeps the calendar readable is written and
//! the round trip through [`unescape`](crate::tree::codec::unescape) is not
//! exact.

use alloc::{borrow::Cow, vec::Vec};

use crate::tree::codec::mode::Escaper;

/// Apply the value escapes by the calendar's escaping mode, over raw bytes.
///
/// RFC 5545 3.3.11 for the modern rules, `;` and a newline for vCalendar 1.0.
/// Borrows when nothing needs escaping; non-UTF-8 content passes through
/// verbatim.
pub(crate) fn escape_with(bytes: &[u8], escaper: Escaper) -> Cow<'_, [u8]> {
    match escaper {
        Escaper::Modern => escape_modern(bytes),
        Escaper::V1_0 => escape_v21(bytes),
    }
}

/// Apply the RFC 5545 3.3.11 value escapes `\\` `\,` `\;` `\n`.
fn escape_modern(bytes: &[u8]) -> Cow<'_, [u8]> {
    if !bytes
        .iter()
        .any(|b| matches!(b, b'\\' | b',' | b';' | b'\n'))
    {
        return Cow::Borrowed(bytes);
    }

    let mut out = Vec::with_capacity(bytes.len());

    for &b in bytes {
        match b {
            b'\\' => out.extend_from_slice(b"\\\\"),
            b',' => out.extend_from_slice(b"\\,"),
            b';' => out.extend_from_slice(b"\\;"),
            b'\n' => out.extend_from_slice(b"\\n"),
            other => out.push(other),
        }
    }

    Cow::Owned(out)
}

/// Apply the vCalendar 1.0 value escapes: `\;`, plus `\n` for a newline.
///
/// Versit has no newline escape, so a newline written into a 1.0 value goes out
/// as `\n` and reads back as those two characters. That is the closest 1.0 can
/// carry it; left raw it would end the line and the calendar would not parse.
fn escape_v21(bytes: &[u8]) -> Cow<'_, [u8]> {
    if !bytes.iter().any(|b| matches!(b, b';' | b'\n')) {
        return Cow::Borrowed(bytes);
    }

    let mut out = Vec::with_capacity(bytes.len() + 2);

    for &b in bytes {
        match b {
            b';' => out.extend_from_slice(b"\\;"),
            b'\n' => out.extend_from_slice(b"\\n"),
            other => out.push(other),
        }
    }

    Cow::Owned(out)
}

#[cfg(test)]
mod tests {
    use alloc::borrow::Cow;

    use crate::tree::codec::{escape::escape_with, mode::Escaper};

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
}
