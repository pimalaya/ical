//! # Unescape (read codec)
//!
//! Resolve the RFC 5545 3.3.11 value escapes when parsing.
//!
//! This is the read half of the escaping codec; its inverse is
//! [`escape`](crate::tree::codec::escape), and the version-specific rules are
//! selected by the [`Escaper`]. The structural decoders in
//! [`decode`](crate::tree::codec::decode) run every value leaf through here.
//!
//! A parameter value is a different alphabet and has its own reader,
//! `unescape_param`. RFC 5545 section 3.2 gives a parameter no backslash
//! escapes at all, which is why RFC 6868 gives it the caret ones instead.

use alloc::{borrow::Cow, string::String, vec::Vec};

use crate::tree::codec::mode::Escaper;

/// Resolve value escapes by the calendar's escaping mode, reading raw value
/// bytes and yielding the decoded text (lossily when the bytes are not UTF-8;
/// the caller keeps the raw bytes on the syntax leaf for fidelity).
pub(crate) fn unescape_with(bytes: &[u8], escaper: Escaper) -> Cow<'_, str> {
    lossy(unescape_bytes(bytes, escaper))
}

/// Resolve value escapes by the calendar's escaping mode at the byte level,
/// preserving any non-UTF-8 content verbatim.
pub(crate) fn unescape_bytes(bytes: &[u8], escaper: Escaper) -> Cow<'_, [u8]> {
    match escaper {
        Escaper::Modern => unescape_modern(bytes),
        Escaper::V1_0 => unescape_v21(bytes),
    }
}

/// Strip the RFC 5545 3.1 delimiters off a parameter value, then resolve the
/// RFC 6868 3.1 encoding: `^n` is a newline, `^^` a caret and `^'` a double
/// quote.
///
/// A caret before anything else, and a trailing one, stay literal: section 3.1
/// forbids reading either as an error. No backslash is touched in any mode,
/// RFC 5545 3.2 giving a parameter value no escapes and RFC 6868 3.2
/// forbidding the backslash ones.
///
/// Borrows when there is nothing to resolve, which is nearly every parameter.
/// A version predating RFC 6868 keeps its carets, and one predating the
/// `quoted-string` production its double quotes: see
/// [`Escaper::has_param_encoding`] and [`Escaper::has_param_quoting`].
pub(crate) fn unescape_param(text: &str, escaper: Escaper) -> Cow<'_, str> {
    // NOTE: RFC 5545 3.1 wraps a value carrying `,`, `;` or `:` in double
    // quotes. The pair is the production's own, never part of what it
    // encloses, so it comes off before the carets are read. An unbalanced one
    // closed nothing and is therefore content.
    let text = match escaper.has_param_quoting() {
        true => text
            .strip_prefix('"')
            .and_then(|inner| inner.strip_suffix('"'))
            .unwrap_or(text),
        false => text,
    };

    if !escaper.has_param_encoding() || !text.contains('^') {
        return Cow::Borrowed(text);
    }

    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();

    while let Some(c) = chars.next() {
        if c != '^' {
            out.push(c);
            continue;
        }

        match chars.peek() {
            Some('n') => out.push('\n'),
            Some('^') => out.push('^'),
            Some('\'') => out.push('"'),
            // NOTE: any other caret sequence is left as it stands, so the
            // caret goes out alone and the character after it is read again.
            _ => {
                out.push('^');
                continue;
            }
        }

        chars.next();
    }

    Cow::Owned(out)
}

/// Interpret unescaped bytes as UTF-8, keeping the borrow when possible.
fn lossy(bytes: Cow<'_, [u8]>) -> Cow<'_, str> {
    match bytes {
        Cow::Borrowed(bytes) => String::from_utf8_lossy(bytes),
        Cow::Owned(bytes) => Cow::Owned(String::from_utf8_lossy(&bytes).into_owned()),
    }
}

/// Resolve the RFC 5545 3.3.11 value escapes `\\` `\,` `\;` `\n`, borrowing
/// when there is nothing to unescape.
fn unescape_modern(bytes: &[u8]) -> Cow<'_, [u8]> {
    if !bytes.contains(&b'\\') {
        return Cow::Borrowed(bytes);
    }

    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] != b'\\' {
            out.push(bytes[i]);
            i += 1;
            continue;
        }

        match bytes.get(i + 1) {
            Some(b'n' | b'N') => out.push(b'\n'),
            Some(&other) => out.push(other),
            None => out.push(b'\\'),
        }
        i += 2;
    }

    Cow::Owned(out)
}

/// Resolve the vCalendar 1.0 value escape `\;` only; a backslash before
/// anything else stays literal.
fn unescape_v21(bytes: &[u8]) -> Cow<'_, [u8]> {
    if !bytes.contains(&b'\\') {
        return Cow::Borrowed(bytes);
    }

    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] != b'\\' {
            out.push(bytes[i]);
            i += 1;
            continue;
        }

        match bytes.get(i + 1) {
            Some(b';') => {
                out.push(b';');
                i += 2;
            }
            Some(&other) => {
                out.push(b'\\');
                out.push(other);
                i += 2;
            }
            None => {
                out.push(b'\\');
                i += 1;
            }
        }
    }

    Cow::Owned(out)
}

#[cfg(test)]
mod tests {
    use alloc::borrow::Cow;

    use crate::tree::codec::unescape::{unescape_param, unescape_with};

    #[test]
    fn unescapes_value_escapes_and_borrows_when_clean() {
        use crate::tree::codec::mode::Escaper;

        assert_eq!(unescape_with(br"a\,b\;c\nd", Escaper::Modern), "a,b;c\nd");
        assert!(matches!(
            unescape_with(b"plain", Escaper::Modern),
            Cow::Borrowed("plain")
        ));
    }

    #[test]
    fn unescapes_the_rfc_6868_parameter_sequences() {
        use crate::tree::codec::mode::Escaper;

        assert_eq!(unescape_param("a^nb^^c^'d", Escaper::Modern), "a\nb^c\"d");
        assert!(matches!(
            unescape_param("plain", Escaper::Modern),
            Cow::Borrowed("plain")
        ));
    }

    #[test]
    fn keeps_an_unknown_caret_sequence_and_a_backslash() {
        use crate::tree::codec::mode::Escaper;

        // NOTE: RFC 6868 section 3.1 forbids reading `^x` as an error, and
        // section 3.2 forbids backslash escaping, so both stay as they are.
        assert_eq!(unescape_param("a^xb^Nc^", Escaper::Modern), "a^xb^Nc^");
        assert_eq!(
            unescape_param(r"C:\temp\note", Escaper::Modern),
            r"C:\temp\note",
        );
    }

    /// RFC 5545 section 3.1 makes the double quotes the `quoted-string`
    /// production's own delimiter, so what comes back is what they enclose.
    #[test]
    fn strips_the_parameter_value_delimiters() {
        use crate::tree::codec::mode::Escaper;

        assert!(matches!(
            unescape_param("\"cid:part1.0001.org\"", Escaper::Modern),
            Cow::Borrowed("cid:part1.0001.org")
        ));
        assert_eq!(unescape_param("\"a^'b\"", Escaper::Modern), "a\"b");
    }

    /// A quote that closes nothing is content, and vCalendar 1.0 has no
    /// quoting at all, so both keep every character they were written with.
    #[test]
    fn keeps_a_quote_that_delimits_nothing() {
        use crate::tree::codec::mode::Escaper;

        assert!(matches!(
            unescape_param("\"CHAIR", Escaper::Modern),
            Cow::Borrowed("\"CHAIR")
        ));
        assert!(matches!(
            unescape_param("\"a,b\"", Escaper::V1_0),
            Cow::Borrowed("\"a,b\"")
        ));
    }

    #[test]
    fn leaves_a_vcalendar_1_0_parameter_caret_alone() {
        use crate::tree::codec::mode::Escaper;

        // NOTE: RFC 6868 updates RFC 5545 alone, so a 1.0 caret is a literal
        // caret and resolving it would corrupt the value.
        assert!(matches!(
            unescape_param("a^nb", Escaper::V1_0),
            Cow::Borrowed("a^nb")
        ));
    }

    #[test]
    fn unescapes_only_the_semicolon_in_v2_1() {
        use crate::tree::codec::{mode::Escaper, unescape::unescape_with};

        // NOTE: vCalendar 1.0 resolves `\;` only; `\n` keeps its literal
        // backslash, and a trailing backslash stays.
        assert_eq!(unescape_with(br"a\;b\nc\", Escaper::V1_0), "a;b\\nc\\");
    }
}
