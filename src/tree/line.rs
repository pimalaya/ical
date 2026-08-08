//! # Content line
//!
//! One raw content line of a card: name, parameters, value, line ending.
//!
//! [`IcalLine`] is the syntactic unit a property occupies. It owns the line
//! tokeniser ([`take`](IcalLine::take), which splits one logical line off the
//! remaining input for [`IcalCst::parse`](crate::tree::cst::IcalCst::parse),
//! unfolding any RFC 5545 3.1 folded continuation lines) and the head splitter
//! that separates the name from its parameters. It exposes its raw value and
//! typed parameter access by lens, but stays generic: the meaning of the name
//! and the decoding of the value belong to the lens markers and the
//! [`decode`](crate::tree::codec::decode) /
//! [`encode`](crate::tree::codec::encode) bridges.
//!
//! Folding, stray blank lines and QUOTED-PRINTABLE soft breaks are resolved on
//! parse, so every layer above sees one logical line, and recorded on the
//! line's [`wire`](IcalLine::wire) shape, so serialization puts them back. A
//! calendar therefore round-trips byte for byte however it was laid out. The
//! final line needs no trailing break.

use core::{fmt, str};

use alloc::{borrow::Cow, string::String, vec, vec::Vec};

use crate::tree::{
    codec::mode::Escaper,
    error::IcalParseError,
    leaf::{IcalLeaf, IcalValueLeaf},
    param::{IcalParamLens, IcalParamNode},
    value::IcalValueNode,
    wire::IcalWire,
};

/// One raw content line: a name, parameters, a value and the line ending.
///
/// This is a *logical* line, not a physical one: [`take`](Self::take) unfolds
/// RFC 5545 3.1 folded continuations and QUOTED-PRINTABLE soft line breaks, so
/// a `IcalLine` never holds an internal line break, only its terminating
/// [`eol`](Self::eol). What it unfolded is kept on [`wire`](Self::wire), which
/// is what puts the folds back on output. It is also the syntactic unit for the
/// `BEGIN` / `VERSION` / `END` envelope lines, not only decoded properties.
#[derive(Clone, Debug)]
pub struct IcalLine<'a> {
    /// The property name leaf, with any group prefix.
    pub name: IcalLeaf<'a>,
    /// The parameters, in source order.
    pub params: Vec<IcalParamNode<'a>>,
    /// The value.
    pub value: IcalValueNode<'a>,
    /// The line ending (`\r\n` or `\n`).
    pub eol: IcalLeaf<'a>,
    /// How the line was laid out on the wire: its folds, the blank lines before
    /// it, its soft breaks. Empty for a built line, and dropped on output once
    /// an edit changes the line's length (see [`IcalWire`]).
    pub wire: IcalWire<'a>,
}

impl<'a> IcalLine<'a> {
    /// Build a property line with a raw text value and the default `\r\n`
    /// ending. Used to seed BEGIN/VERSION/END and to encode simple values.
    pub fn text(name: impl Into<Cow<'a, str>>, value: impl Into<Cow<'a, str>>) -> Self {
        Self {
            name: IcalLeaf(name.into()),
            params: Vec::new(),
            value: IcalValueNode::from_components(
                vec![vec![IcalValueLeaf::from(value.into())]],
                Escaper::Modern,
            ),
            eol: IcalLeaf(Cow::Borrowed("\r\n")),
            wire: IcalWire::default(),
        }
    }

    /// Tokenise the logical line at the start of `rest`, unfolding any folded
    /// continuation lines, and return it with the remaining input. RFC 5545 3.1
    /// folds a long line by inserting a CRLF and a single leading space or tab;
    /// unfolding drops them. A line with no folds borrows the source; a folded
    /// line is rebuilt owned, since its bytes are no longer contiguous.
    pub fn take(rest: &'a [u8]) -> Result<(Self, &'a [u8]), IcalParseError> {
        // NOTE: Everything this tokeniser resolves is recorded here, so
        // serialization can put it back.
        let mut wire = IcalWire::default();

        // NOTE: skip blank lines: real-world exports sometimes emit them.
        let mut head = rest;
        let (first, eol, mut tail) = loop {
            if head.is_empty() {
                return Err(IcalParseError::MissingCrlf(lossy(rest)));
            }
            let (content, eol, next) = physical_line(head);
            if content.is_empty() {
                head = next;
                continue;
            }
            break (content, eol, next);
        };

        if head.len() < rest.len() {
            wire.skipped(0, ascii(&rest[..rest.len() - head.len()]));
        }

        // NOTE: A line that begins with folding whitespace but has no line to
        // continue (a dangling continuation, e.g. left after a dropped blank
        // line) would fold into the previous line on reparse; strip the leading
        // whitespace so it stays its own line, and record it so it still
        // round-trips.
        let indented = first;
        let first = strip_leading_wsp(first);

        if first.len() < indented.len() {
            wire.skipped(0, ascii(&indented[..indented.len() - first.len()]));
        }

        // NOTE: QUOTED-PRINTABLE soft line breaks: a line whose head declares
        // ENCODING=QUOTED-PRINTABLE and whose value ends with `=` continues on
        // the next physical line. Param-driven, so it applies to any version's
        // card that uses the encoding, not just 2.1.
        if first.ends_with(b"=") && head_is_quoted_printable(first) {
            let mut logical = Vec::from(&first[..first.len() - 1]);
            wire.soft(logical.len(), is_crlf(eol));

            let mut last_eol;
            loop {
                let (continuation, eol, next) = physical_line(tail);
                last_eol = eol;
                tail = next;
                match continuation.strip_suffix(b"=") {
                    Some(head) => {
                        logical.extend_from_slice(head);
                        if tail.is_empty() {
                            // NOTE: The last continuation ends with a
                            // soft-break marker and nothing follows: the `=` is
                            // on the wire, the break after it is the line's own
                            // ending.
                            wire.skipped(logical.len(), "=");
                            break;
                        }
                        wire.soft(logical.len(), is_crlf(eol));
                    }
                    None => {
                        logical.extend_from_slice(continuation);
                        break;
                    }
                }
            }

            let mut line = Self::parse(&logical, b"")?.into_static();
            line.eol = eol_leaf(last_eol);
            line.wire.prepend(wire.into_static());
            return Ok((line, tail));
        }

        if !starts_with_wsp(tail) {
            let mut line = Self::parse(first, eol)?;
            line.wire.prepend(wire);
            return Ok((line, tail));
        }

        let mut logical = Vec::from(first);
        let mut last_eol = eol;

        while starts_with_wsp(tail) {
            let (continuation, eol, next) = physical_line(&tail[1..]);
            wire.fold(logical.len(), is_crlf(last_eol), tail[0]);
            logical.extend_from_slice(continuation);
            last_eol = eol;
            tail = next;
        }

        let mut line = Self::parse(&logical, b"")?.into_static();
        line.eol = eol_leaf(last_eol);
        line.wire.prepend(wire.into_static());

        Ok((line, tail))
    }

    /// Split the first physical line off `rest`, verbatim (its content and its
    /// ending), and return it with what follows.
    ///
    /// This is the recovering parser's step over a line [`take`](Self::take)
    /// refuses: the bytes are kept whole rather than structured, so they still
    /// round-trip.
    pub fn take_physical(rest: &'a [u8]) -> (&'a [u8], &'a [u8]) {
        let (content, eol, tail) = physical_line(rest);
        (&rest[..content.len() + eol.len()], tail)
    }

    /// Convert into an owned line whose every leaf is owned (`'static`).
    pub(crate) fn into_static(self) -> IcalLine<'static> {
        IcalLine {
            name: self.name.into_static(),
            params: self
                .params
                .into_iter()
                .map(IcalParamNode::into_static)
                .collect(),
            value: self.value.into_static(),
            eol: self.eol.into_static(),
            wire: self.wire.into_static(),
        }
    }

    /// The raw bytes of the line's first value, for simple single-value lines.
    pub fn raw_value(&self) -> &[u8] {
        self.value.first_value_bytes()
    }

    /// The raw first value as UTF-8 text, lossily; for the ASCII envelope
    /// values (`VERSION`) and diagnostics.
    pub fn raw_value_str(&self) -> Cow<'_, str> {
        String::from_utf8_lossy(self.value.first_value_bytes())
    }

    /// Serialize the whole line to bytes, exactly as parsed: its logical
    /// content, laid back out in the wire shape it arrived in.
    pub(crate) fn write_bytes(&self, out: &mut Vec<u8>) {
        if self.wire.is_empty() {
            self.write_logical(out);
        } else {
            let mut logical = Vec::new();
            self.write_logical(&mut logical);
            self.wire.write_bytes(&logical, out);
        }

        out.extend_from_slice(self.eol.get().as_bytes());
    }

    /// Serialize the logical line (its name, parameters and value), with no
    /// line ending and no wire shape. This is the byte string the wire offsets
    /// index.
    fn write_logical(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(self.name.get().as_bytes());

        for param in &self.params {
            out.push(b';');
            param.write_bytes(out);
        }

        out.push(b':');
        self.value.write_bytes(out);
    }

    /// The first parameter of type `P`, decoded.
    pub fn param<P: IcalParamLens>(&self) -> Option<P::Target<'_>> {
        self.params
            .iter()
            .find(|param| param.name.get().eq_ignore_ascii_case(&P::KIND))
            .map(|param| P::decode(param))
    }

    /// The first parameter of type `P`, mutably (raw, for editing its leaves).
    pub fn param_mut<P: IcalParamLens>(&mut self) -> Option<&mut IcalParamNode<'a>> {
        self.params
            .iter_mut()
            .find(|param| param.name.get().eq_ignore_ascii_case(&P::KIND))
    }

    /// Split one logical line into a typed line at the colon, separating the
    /// name, its parameters and the value. The head (name and parameters) must
    /// be valid UTF-8, as every version's grammar guarantees; only the value
    /// may carry a foreign charset, so it is kept as raw bytes.
    fn parse<'b>(content: &'b [u8], eol: &'b [u8]) -> Result<IcalLine<'b>, IcalParseError> {
        let Some(colon) = memchr::memchr(b':', content) else {
            return Err(IcalParseError::MissingPropertyColon(lossy(content)));
        };

        let head = str::from_utf8(&content[..colon])
            .map_err(|_| IcalParseError::NonUtf8Header(lossy(&content[..colon])))?;
        let (name, params) = split_head(head);

        let mut value = &content[colon + 1..];
        let mut wire = IcalWire::default();

        // NOTE: A QUOTED-PRINTABLE value ending in `=` is a dangling soft-break
        // marker, however it got there (a soft-break join, a folded
        // continuation, or raw input): valid content would encode a literal `=`
        // as `=3D`. Left in, it would re-trigger soft-break joining on reparse
        // and swallow the next line, so the logical line drops it and the wire
        // shape keeps it. This never touches base64 padding, since
        // `ENCODING=BASE64` is not quoted-printable.
        if head_is_quoted_printable(content) {
            let full = value.len();
            while value.last() == Some(&b'=') {
                value = &value[..value.len() - 1];
            }
            if value.len() < full {
                let end = colon + 1 + value.len();
                wire.skipped(end, ascii(&content[end..colon + 1 + full]));
            }
        }

        wire.seal(colon + 1 + value.len());

        Ok(IcalLine {
            name: IcalLeaf::from(name),
            params,
            value: IcalValueNode::parse(value),
            eol: IcalLeaf::from(str::from_utf8(eol).unwrap_or("")),
            wire,
        })
    }
}

impl fmt::Display for IcalLine<'_> {
    /// The line as text, wire shape included, lossily for a non-UTF-8 value.
    /// `IcalLine::write_bytes` is the byte-faithful path.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.wire.is_empty() {
            f.write_str(self.name.get())?;

            for param in &self.params {
                write!(f, ";{param}")?;
            }

            return write!(f, ":{}{}", self.value, self.eol.get());
        }

        let mut bytes = Vec::new();
        self.write_bytes(&mut bytes);
        f.write_str(&String::from_utf8_lossy(&bytes))
    }
}

/// Split a head into its name and its `;`-separated parameters.
fn split_head(head: &str) -> (&str, Vec<IcalParamNode<'_>>) {
    let (name, mut rest) = match head.find(';') {
        Some(semi) => (&head[..semi], &head[semi..]),
        None => return (head, Vec::new()),
    };

    let mut params = Vec::new();

    while let Some(after) = rest.strip_prefix(';') {
        let (param, tail) = match after.find(';') {
            Some(semi) => (&after[..semi], &after[semi..]),
            None => (after, ""),
        };

        params.push(IcalParamNode::parse(param));
        rest = tail;
    }

    (name, params)
}

/// Split the first physical line off `rest`: its content (without the line
/// ending), its line ending, and the remaining input. A final line with no
/// trailing break is taken whole, with an empty ending.
fn physical_line(rest: &[u8]) -> (&[u8], &[u8], &[u8]) {
    let Some(lf) = memchr::memchr(b'\n', rest) else {
        return (rest, b"", b"");
    };

    let tail = &rest[lf + 1..];

    let (content, eol) = if lf > 0 && rest[lf - 1] == b'\r' {
        (&rest[..lf - 1], &rest[lf - 1..lf + 1])
    } else {
        (&rest[..lf], &rest[lf..lf + 1])
    };

    (content, eol, tail)
}

/// Whether `rest` begins with a folding whitespace (space or tab).
fn starts_with_wsp(rest: &[u8]) -> bool {
    matches!(rest.first(), Some(b' ' | b'\t'))
}

/// Strip any leading folding whitespace (space, tab) or stray line-break byte
/// (`\r`, `\n`) from a line's content, so its name never begins with a byte
/// that another layer (folding, blank-line skipping) would re-strip on reparse.
fn strip_leading_wsp(mut bytes: &[u8]) -> &[u8] {
    while matches!(bytes.first(), Some(b' ' | b'\t' | b'\r' | b'\n')) {
        bytes = &bytes[1..];
    }
    bytes
}

/// Whether a line's head (its name and parameters, before the `:`) declares the
/// `QUOTED-PRINTABLE` encoding, as an `ENCODING=` parameter or a bare token.
fn head_is_quoted_printable(line: &[u8]) -> bool {
    let head = match memchr::memchr(b':', line) {
        Some(colon) => &line[..colon],
        None => return false,
    };

    head.split(|&b| b == b';').any(|token| {
        token.eq_ignore_ascii_case(b"QUOTED-PRINTABLE")
            || token.eq_ignore_ascii_case(b"ENCODING=QUOTED-PRINTABLE")
    })
}

/// An owned line-ending leaf from raw bytes (always an ASCII `\r\n` / `\n`).
fn eol_leaf(bytes: &[u8]) -> IcalLeaf<'static> {
    IcalLeaf::from(String::from_utf8_lossy(bytes).into_owned())
}

/// Whether a line ending is a `\r\n` rather than a bare `\n`.
fn is_crlf(eol: &[u8]) -> bool {
    eol.starts_with(b"\r")
}

/// Bytes the tokeniser resolved away, as text. Every one of them is a line
/// break, a space, a tab or an `=`, so the conversion never fails; a lone `""`
/// on the impossible path keeps this total rather than panicking.
fn ascii(bytes: &[u8]) -> &str {
    str::from_utf8(bytes).unwrap_or("")
}

/// A lossy owned string of raw bytes, for error diagnostics.
fn lossy(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString;

    use crate::tree::{line::IcalLine, value::IcalValueNode};

    #[test]
    fn takes_one_line_and_leaves_the_rest() {
        let (line, rest) = IcalLine::take(b"FN:John\r\nEND:VCALENDAR\r\n").unwrap();
        assert_eq!(line.name.get(), "FN");
        assert_eq!(line.to_string(), "FN:John\r\n");
        assert_eq!(rest, b"END:VCALENDAR\r\n");
    }

    #[test]
    fn splits_parameters_off_the_head_then_round_trips() {
        let (line, _) = IcalLine::take(b"TEL;TYPE=work,home:123\r\n").unwrap();
        assert_eq!(line.params.len(), 1);
        assert_eq!(line.to_string(), "TEL;TYPE=work,home:123\r\n");
    }

    #[test]
    fn accepts_a_bare_lf_ending() {
        let (line, _) = IcalLine::take(b"FN:John\n").unwrap();
        assert_eq!(line.to_string(), "FN:John\n");
    }

    #[test]
    fn unfolds_space_and_tab_continuations() {
        let (line, rest) =
            IcalLine::take(b"NOTE:foo\r\n bar\r\n\tbaz\r\nEND:VCALENDAR\r\n").unwrap();
        assert_eq!(line.name.get(), "NOTE");
        assert_eq!(line.raw_value_str(), "foobarbaz");
        assert_eq!(rest, b"END:VCALENDAR\r\n");
    }

    #[test]
    fn serializes_a_folded_line_back_folded() {
        let (line, _) = IcalLine::take(b"NOTE:foo\r\n bar\r\n").unwrap();
        assert_eq!(line.raw_value_str(), "foobar");
        assert_eq!(line.to_string(), "NOTE:foo\r\n bar\r\n");
    }

    #[test]
    fn keeps_the_folding_whitespace_and_the_break_it_arrived_with() {
        let (line, _) = IcalLine::take(b"NOTE:foo\n\tbar\r\n").unwrap();
        assert_eq!(line.to_string(), "NOTE:foo\n\tbar\r\n");
    }

    #[test]
    fn serializes_a_skipped_blank_line_back() {
        let (line, _) = IcalLine::take(b"\r\n\r\nFN:John\r\n").unwrap();
        assert_eq!(line.to_string(), "\r\n\r\nFN:John\r\n");
    }

    #[test]
    fn drops_the_fold_points_once_the_value_is_edited() {
        // NOTE: The old offsets index bytes that are no longer there, so the
        // edited line goes out unfolded rather than folded in the wrong places.
        let (mut line, _) = IcalLine::take(b"NOTE:foo\r\n bar\r\n").unwrap();
        line.value = IcalValueNode::parse(b"something else entirely");
        assert_eq!(line.to_string(), "NOTE:something else entirely\r\n");
    }

    #[test]
    fn keeps_the_fold_points_when_an_edit_keeps_the_length() {
        // NOTE: Same length, so every offset still indexes what it did: the
        // line is folded exactly where it was.
        let (mut line, _) = IcalLine::take(b"NOTE:foo\r\n bar\r\n").unwrap();
        line.value = IcalValueNode::parse(b"BARFOO");
        assert_eq!(line.to_string(), "NOTE:BAR\r\n FOO\r\n");
    }

    #[test]
    fn keeps_whitespace_beyond_the_single_fold_indicator() {
        // NOTE: only the first space is the fold marker; the rest is value
        // content.
        let (line, _) = IcalLine::take(b"NOTE:foo\r\n  bar\r\n").unwrap();
        assert_eq!(line.raw_value_str(), "foo bar");
    }

    #[test]
    fn skips_blank_lines_before_the_next_line() {
        let (line, rest) = IcalLine::take(b"\r\n\r\nFN:John\r\nEND:VCALENDAR\r\n").unwrap();
        assert_eq!(line.name.get(), "FN");
        assert_eq!(rest, b"END:VCALENDAR\r\n");
    }

    #[test]
    fn tolerates_a_missing_final_line_break() {
        let (line, rest) = IcalLine::take(b"END:VCALENDAR").unwrap();
        assert_eq!(line.name.get(), "END");
        assert_eq!(line.to_string(), "END:VCALENDAR");
        assert_eq!(rest, b"");
    }

    #[test]
    fn joins_a_quoted_printable_soft_broken_line() {
        // NOTE: Two soft breaks: the first continuation itself ends with `=`
        // (the Some arm), the second does not (the None arm).
        let (line, _) = IcalLine::take(
            b"NOTE;ENCODING=QUOTED-PRINTABLE:caf=\r\n=C3=\r\n=A9\r\nEND:VCALENDAR\r\n",
        )
        .unwrap();
        assert_eq!(line.name.get(), "NOTE");
        assert_eq!(line.raw_value_str(), "caf=C3=A9");
        assert_eq!(line.raw_value(), b"caf=C3=A9");
    }

    #[test]
    fn errors_when_there_is_no_content_line() {
        assert!(IcalLine::take(b"").is_err());
        assert!(IcalLine::take(b"\r\n\r\n").is_err());
    }

    #[test]
    fn rejects_a_non_utf8_head() {
        let mut raw = b"X-".to_vec();
        raw.push(0xff);
        raw.extend_from_slice(b":v\r\n");
        assert!(IcalLine::take(&raw).is_err());
    }

    #[test]
    fn finds_a_parameter_mutably() {
        use crate::tree::param::language::LANGUAGE;

        let (mut line, _) = IcalLine::take(b"SUMMARY;LANGUAGE=en:Lunch\r\n").unwrap();
        assert!(line.param_mut::<LANGUAGE>().is_some());
    }

    #[test]
    fn a_trailing_equals_without_a_colon_is_not_quoted_printable() {
        // NOTE: `abc=` ends with `=` but has no colon, so the QP soft-break
        // check bails and the line then fails for want of a value separator.
        assert!(IcalLine::take(b"abc=\r\n").is_err());
    }

    #[test]
    fn quoted_printable_join_stops_at_an_empty_tail() {
        // NOTE: The final continuation ends with `=` and nothing follows, so
        // the join loop exits via the empty-tail guard rather than a non-`=`
        // line.
        let (line, rest) = IcalLine::take(b"NOTE;ENCODING=QUOTED-PRINTABLE:a=\r\nb=\r\n").unwrap();
        assert_eq!(line.raw_value_str(), "ab");
        assert_eq!(rest, b"");
    }
}
