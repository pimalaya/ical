//! # Content line
//!
//! One raw content line of a calendar: name, parameters, value, line ending.
//!
//! [`IcalLine`] is the syntactic unit a property occupies. It owns the line
//! tokeniser ([`take`](IcalLine::take), which splits one logical line off the
//! remaining input) and the head splitter separating the name from its
//! parameters.
//!
//! It stays generic: what the name means and how the value decodes belong to
//! the property lenses and the [`codec`](crate::tree::codec).
//!
//! Folding, stray blank lines and QUOTED-PRINTABLE soft breaks are resolved
//! on parse, so every layer above sees one logical line, and recorded on the
//! line's [`wire`](IcalLine::wire) shape, so serialization puts them back.
//!
//! A calendar therefore round-trips byte for byte however it was laid out.
//! The final line needs no trailing break.

use core::{fmt, mem};

use alloc::{borrow::Cow, string::String, vec, vec::Vec};

use crate::tree::{
    codec::{
        escape::position,
        mode::Escaper,
        unescape::{text, utf8_lossy},
    },
    error::IcalParseError,
    leaf::{IcalLeaf, IcalValueLeaf, put},
    param::{lens::IcalParamLens, node::IcalParamNode},
    value::node::IcalValueNode,
    wire::{FOLD_OCTETS, IcalWire, fold},
};

/// One raw content line: a name, parameters, a value and the line ending.
///
/// This is a *logical* line: [`take`](Self::take) unfolds RFC 5545 3.1
/// continuations and QUOTED-PRINTABLE soft breaks, so a line holds no internal
/// break, only its terminating [`eol`](Self::eol), and what it unfolded is
/// kept on [`wire`](Self::wire) to put the folds back on output.
///
/// It is also the syntactic unit for the `BEGIN` / `VERSION` / `END` envelope
/// lines, not only decoded properties.
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

    /// Tokenise the logical line at the start of `rest`, unfolding as it goes.
    ///
    /// Returns the line and the remaining input. RFC 5545 3.1 folds a long line
    /// by inserting a CRLF and a single leading space or tab; unfolding drops
    /// them. A line with no folds borrows the source; a folded one owns its
    /// joined value, its bytes no longer contiguous, and borrows its head when
    /// its colon sits on its first physical line.
    pub fn take(rest: &'a [u8]) -> Result<(Self, &'a [u8]), IcalParseError> {
        let mut line = Self::empty();
        let tail = Self::take_into(rest, &mut line)?;
        Ok((line, tail))
    }

    /// [`take`](Self::take) the line into `slot`, returning the rest.
    ///
    /// A component grows its item list by an [`empty`](Self::empty) line and
    /// fills it here: a line returned by value is staged on the stack and
    /// copied, which stalls the loads that follow.
    ///
    /// `slot` must be an [`empty`](Self::empty) line: its fields are written
    /// over without being dropped, there being nothing in them to free. It is
    /// left empty on an error.
    pub(crate) fn take_into(rest: &'a [u8], slot: &mut Self) -> Result<&'a [u8], IcalParseError> {
        let (first, eol, tail) = physical_line(rest);

        // NOTE: The common line is one physical line, neither blank nor
        // indented, soft-broken or folded, so its wire shape is its logical
        // shape. It is written into the slot field by field, every value
        // passed in registers: a line staged in parts and moved into place
        // waits on every part's stores.
        let plain = first
            .first()
            .is_some_and(|&byte| !matches!(byte, b' ' | b'\t' | b'\r'))
            && !starts_with_wsp(tail)
            && !(first.ends_with(b"=") && head_is_quoted_printable(first));

        if !plain {
            return Self::take_wired(rest, slot);
        }

        let (colon, semi) = match head_marks(rest, first.len()) {
            Some((colon, semi)) => (colon, Some(semi)),
            None => match value_colon(first) {
                Some(colon) => (colon, None),
                None => return Err(IcalParseError::MissingPropertyColon(lossy(first))),
            },
        };

        let head = &first[..colon];

        let Some(head_str) = text(head) else {
            return Err(IcalParseError::NonUtf8Header(lossy(head)));
        };

        let semi = semi.unwrap_or_else(|| param_semicolon(head_str));
        let name = take_params(head_str, semi, &mut slot.params);

        // NOTE: The empty line's fields hold no allocation, so forgetting
        // them saves a drop per field.
        mem::forget(mem::replace(&mut slot.name, IcalLeaf::from(name)));
        slot.value.hold(&first[colon + 1..]);
        mem::forget(mem::replace(&mut slot.eol, eol_leaf(eol)));
        slot.wire.seal(first.len());

        Ok(tail)
    }

    /// [`take_into`](Self::take_into) for a line whose wire shape is not its
    /// logical shape: blank lines before it, an indented first line, folds
    /// or soft breaks, each recorded on its wire shape.
    fn take_wired(rest: &'a [u8], slot: &mut Self) -> Result<&'a [u8], IcalParseError> {
        // NOTE: Everything this tokeniser resolves is recorded here, so
        // serialization can put it back.
        let mut wire = IcalWire::new();

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
        // calendar that uses the encoding, not just 2.1.
        let soft = first.ends_with(b"=") && head_is_quoted_printable(first);

        if !soft && !starts_with_wsp(tail) {
            let Some(colon) = value_colon(first) else {
                return Err(IcalParseError::MissingPropertyColon(lossy(first)));
            };

            Self::build(
                slot,
                &first[..colon],
                Cow::Borrowed(&first[colon + 1..]),
                eol_leaf(eol),
                wire,
            )?;

            return Ok(tail);
        }

        // NOTE: The line goes on past its first physical line. With the colon
        // on that first line (outside quotes, the scan that finds the same
        // colon on the joined line), the head borrows the input and only the
        // value is joined, once, into the buffer the line keeps. Otherwise the
        // whole logical line is joined, parsed, then copied out.
        let colon = unquoted_colon(first);
        let base = colon.map_or(0, |colon| colon + 1);
        let mut logical = Vec::new();
        let last_eol;

        if soft {
            logical.extend_from_slice(&first[base..first.len() - 1]);
            wire.soft(base + logical.len(), is_crlf(eol));

            loop {
                let (continuation, eol, next) = physical_line(tail);
                tail = next;

                match continuation.strip_suffix(b"=") {
                    Some(head) => {
                        logical.extend_from_slice(head);
                        if tail.is_empty() {
                            // NOTE: The last continuation ends with a
                            // soft-break marker and nothing follows: the `=` is
                            // on the wire, the break after it is the line's own
                            // ending.
                            wire.skipped(base + logical.len(), "=");
                            last_eol = eol;
                            break;
                        }
                        wire.soft(base + logical.len(), is_crlf(eol));
                    }
                    None => {
                        logical.extend_from_slice(continuation);
                        last_eol = eol;
                        break;
                    }
                }
            }
        } else {
            // NOTE: Each continuation is found once, its fold recorded and its
            // length counted, so a long folded value is sized before its one
            // copy, and the copy takes each continuation's length from the
            // offset of the fold after it rather than scanning again.
            let (mut len, mut eol, mut next) = (first.len(), eol, tail);

            while starts_with_wsp(next) {
                let (continuation, next_eol, rest) = physical_line(&next[1..]);
                wire.fold(len, is_crlf(eol), next[0]);
                len += continuation.len();
                eol = next_eol;
                next = rest;
            }

            last_eol = eol;
            logical.reserve_exact(len - base);
            logical.extend_from_slice(&first[base..]);

            let mut folds = wire.folds().peekable();

            while let Some((offset, _)) = folds.next() {
                let (end, eol) = match folds.peek() {
                    Some(&(end, crlf)) => (end, if crlf { 2 } else { 1 }),
                    None => (len, last_eol.len()),
                };

                logical.extend_from_slice(&tail[1..1 + end - offset]);
                tail = &tail[1 + end - offset + eol..];
            }

            debug_assert_eq!(tail.len(), next.len());
        }

        match colon {
            Some(colon) => Self::build(
                slot,
                &first[..colon],
                Cow::Owned(logical),
                eol_leaf(last_eol),
                wire,
            )?,
            None => {
                let mut line = Self::parse(&logical)?.into_static();
                line.eol = eol_leaf(last_eol);
                line.wire.prepend(wire);
                *slot = line;
            }
        }

        Ok(tail)
    }

    /// A placeholder line for [`take_into`](Self::take_into) to fill in place.
    pub(crate) const fn empty() -> Self {
        Self {
            name: IcalLeaf(Cow::Borrowed("")),
            params: Vec::new(),
            value: IcalValueNode::from_components(Vec::new(), Escaper::Modern),
            eol: IcalLeaf(Cow::Borrowed("")),
            wire: IcalWire::new(),
        }
    }

    /// Split the first physical line off `rest` verbatim, content and ending.
    ///
    /// Returned with what follows. This is the recovering parser's step over a
    /// line [`take`](Self::take) refuses: the bytes are kept whole rather than
    /// structured, so they still round-trip.
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

    /// The line's whole value as it is written on the wire: still escaped, not
    /// transfer-decoded, every `;` and `,` in place.
    ///
    /// Raw, so for a value to read, decode it instead: the value cursor or
    /// [`IcalLine::decode`].
    pub fn raw_value(&self) -> Cow<'_, [u8]> {
        self.value.raw_bytes()
    }

    /// The whole raw value as UTF-8 text, lossily, as [`raw_value`] reads it.
    ///
    /// [`raw_value`]: Self::raw_value
    pub fn raw_value_str(&self) -> Cow<'_, str> {
        utf8_lossy(self.value.raw_bytes())
    }

    /// Serialize the whole line to bytes: its logical content (its name,
    /// parameters and value, the byte string the wire offsets index), laid
    /// back out in the wire shape it arrived in, or folded at 75 octets when
    /// it has none that still fits it (see [`IcalWire`]).
    ///
    /// An unsplit value is laid out straight from its node, so a long folded
    /// value is copied once.
    pub(crate) fn write_bytes(&self, out: &mut Vec<u8>) {
        let start = out.len();

        put(out, self.name.get().as_bytes());

        for param in &self.params {
            out.push(b';');
            param.write_bytes(out);
        }

        out.push(b':');

        match self.value.unsplit_bytes() {
            Some(value) if self.wire.fits(out.len() - start + value.len()) => {
                self.wire.lay_out_value(out, start, value);
            }
            _ => {
                self.value.write_bytes(out);

                if self.wire.fits(out.len() - start) {
                    self.wire.lay_out(out, start);
                } else {
                    fold_long(out, start, self.eol.get() != "\n");
                }
            }
        }

        put(out, self.eol.get().as_bytes());
    }

    /// About how many bytes [`write_bytes`](Self::write_bytes) writes: exact
    /// for an unedited parsed line folded with `\r\n`, a guess otherwise.
    pub(crate) fn size_hint(&self) -> usize {
        self.wire.size_hint() + self.eol.get().len()
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
    fn parse(content: &[u8]) -> Result<IcalLine<'_>, IcalParseError> {
        let Some(colon) = value_colon(content) else {
            return Err(IcalParseError::MissingPropertyColon(lossy(content)));
        };

        let mut line = IcalLine::empty();

        IcalLine::build(
            &mut line,
            &content[..colon],
            Cow::Borrowed(&content[colon + 1..]),
            IcalLeaf(Cow::Borrowed("")),
            IcalWire::new(),
        )?;

        Ok(line)
    }

    /// Fill `slot` from a line's head (the name and parameters before the
    /// colon), its value bytes, its line ending and the wire shape the
    /// tokeniser recorded, in offset order.
    ///
    /// `slot` is an [`empty`](Self::empty) line, untouched on an error.
    fn build(
        slot: &mut Self,
        head: &'a [u8],
        mut value: Cow<'a, [u8]>,
        eol: IcalLeaf<'a>,
        mut wire: IcalWire<'a>,
    ) -> Result<(), IcalParseError> {
        let head_str = text(head).ok_or_else(|| IcalParseError::NonUtf8Header(lossy(head)))?;

        let base = head.len() + 1;

        // NOTE: A QUOTED-PRINTABLE value ending in `=` is a dangling soft-break
        // marker, however it got there (a soft-break join, a folded
        // continuation, or raw input): valid content would encode a literal `=`
        // as `=3D`. Left in, it would re-trigger soft-break joining on reparse
        // and swallow the next line, so the logical line drops it and the wire
        // shape keeps it. This never touches base64 padding, since
        // `ENCODING=BASE64` is not quoted-printable.
        if value.last() == Some(&b'=') && head_declares_quoted_printable(head) {
            let full = value.len();
            let kept = full - value.iter().rev().take_while(|&&b| b == b'=').count();

            match &mut value {
                Cow::Borrowed(bytes) => *bytes = &bytes[..kept],
                Cow::Owned(bytes) => bytes.truncate(kept),
            }

            wire.insert_skipped_equals(base + kept, full - kept);
        }

        wire.seal(base + value.len());

        let name = take_params(head_str, param_semicolon(head_str), &mut slot.params);

        // NOTE: The empty line's fields hold no allocation, so forgetting
        // them saves a drop per field, the value's an out-of-line call.
        mem::forget(mem::replace(&mut slot.name, IcalLeaf::from(name)));
        mem::forget(mem::replace(
            &mut slot.value,
            IcalValueNode::parse_cow(value),
        ));
        mem::forget(mem::replace(&mut slot.eol, eol));
        mem::forget(mem::replace(&mut slot.wire, wire));

        Ok(())
    }
}

impl fmt::Display for IcalLine<'_> {
    /// The line as text, wire shape included, lossily for a non-UTF-8 value.
    /// `IcalLine::write_bytes` is the byte-faithful path.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.write_text(f, &mut Vec::new())
    }
}

impl IcalLine<'_> {
    /// Write the line into `f` as `Display` does, laid out in `buffer` first,
    /// whatever it held.
    pub(crate) fn write_text(
        &self,
        f: &mut fmt::Formatter<'_>,
        buffer: &mut Vec<u8>,
    ) -> fmt::Result {
        buffer.clear();
        self.write_bytes(buffer);

        match text(buffer) {
            Some(text) => f.write_str(text),
            None => f.write_str(&String::from_utf8_lossy(buffer)),
        }
    }
}

/// Fold the logical line written in `out` from `start` on at 75 octets when it
/// runs longer, as a line with no recorded shape is laid out.
///
/// A `QUOTED-PRINTABLE` line is never folded: a fold after one of its `=` would
/// read back as a soft break, and the encoding has soft breaks of its own for
/// a long value.
pub(crate) fn fold_long(out: &mut Vec<u8>, start: usize, crlf: bool) {
    if out.len() - start > FOLD_OCTETS && !head_is_quoted_printable(&out[start..]) {
        fold(out, start, crlf);
    }
}

/// Split a head's `;`-separated parameters into `params`, each built where it
/// lands, and return its name, `semi` being the head's
/// [first semicolon](param_semicolon).
fn take_params<'a>(
    head: &'a str,
    semi: Option<usize>,
    params: &mut Vec<IcalParamNode<'a>>,
) -> &'a str {
    let Some(semi) = semi else {
        return head;
    };

    let mut rest = Some(&head[semi + 1..]);

    while let Some(after) = rest {
        rest = IcalParamNode::take_into(after, params);
    }

    &head[..semi]
}

/// The value colon of the line at the start of `rest`, the input from the line
/// on, and the semicolon ending its name, if any, when no quote comes before
/// the colon: the head has no quoted value then, so the first `:` is the
/// value colon and the first `;` before it ends the name. `None` otherwise,
/// for [`value_colon`] and [`param_semicolon`] to settle.
///
/// `len` is the length of the line's first physical line. Both bytes are found
/// sixteen at a time, rather than byte by byte, over the line and the bytes
/// after it up to a whole block, so a short line still gets whole blocks and
/// a line with no colon is never scanned past.
#[inline(always)]
fn head_marks(rest: &[u8], len: usize) -> Option<(usize, Option<usize>)> {
    let rest = &rest[..rest.len().min(len.next_multiple_of(16))];
    let at = position(rest, |byte| {
        (byte == b':') | (byte == b';') | (byte == b'"')
    })?;

    if at >= len {
        return None;
    }

    match rest[at] {
        b':' => Some((at, None)),
        b';' => {
            let after = &rest[at + 1..];
            let colon = at + 1 + position(after, |byte| (byte == b':') | (byte == b'"'))?;
            (colon < len && rest[colon] == b':').then_some((colon, Some(at)))
        }
        _ => None,
    }
}

/// The index of the `:` separating a line's head from its value.
///
/// The first one outside a double-quoted parameter value, which RFC 5545 3.2
/// lets carry both a colon and a semicolon. A head with an unbalanced quote
/// would swallow the rest of the line, so with no colon outside quotes the
/// scan falls back to the first colon anywhere and the line still parses.
fn value_colon(content: &[u8]) -> Option<usize> {
    unquoted_colon(content).or_else(|| memchr::memchr(b':', content))
}

/// The index of the first `:` outside a double-quoted parameter value, with no
/// fallback.
///
/// The colon and the quotes are found by block scans, a quote skipping to the
/// one closing it, rather than byte by byte.
fn unquoted_colon(content: &[u8]) -> Option<usize> {
    let mut at = 0;

    loop {
        at += position(&content[at..], |byte| (byte == b':') | (byte == b'"'))?;

        if content[at] == b':' {
            return Some(at);
        }

        at += 1 + position(&content[at + 1..], |byte| byte == b'"')? + 1;
    }
}

/// The byte index of the first `;` of a head that sits outside a double-quoted
/// parameter value, the one separating one parameter from the next.
fn param_semicolon(head: &str) -> Option<usize> {
    let mut quoted = false;

    for (i, byte) in head.bytes().enumerate() {
        match byte {
            b'"' => quoted = !quoted,
            b';' if !quoted => return Some(i),
            _ => {}
        }
    }

    None
}

/// Split the first physical line off `rest`: its content (without the line
/// ending), its line ending, and the remaining input. A final line with no
/// trailing break is taken whole, with an empty ending.
///
/// The line feed is found sixteen bytes at a time by an inlined scan over the
/// rest of the input: a call a line paid more than it searched, the lines
/// running some 21 bytes.
fn physical_line(rest: &[u8]) -> (&[u8], &[u8], &[u8]) {
    let Some(lf) = position(rest, |byte| byte == b'\n') else {
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
    match value_colon(line) {
        Some(colon) => head_declares_quoted_printable(&line[..colon]),
        None => false,
    }
}

/// Whether a head (a line's bytes before its value colon) declares the
/// `QUOTED-PRINTABLE` encoding.
fn head_declares_quoted_printable(head: &[u8]) -> bool {
    head.split(|&b| b == b';').any(|token| {
        token.eq_ignore_ascii_case(b"QUOTED-PRINTABLE")
            || token.eq_ignore_ascii_case(b"ENCODING=QUOTED-PRINTABLE")
    })
}

/// The line-ending leaf of raw line-ending bytes, which the splitter only
/// ever cuts as `\r\n`, `\n` or nothing.
fn eol_leaf(bytes: &[u8]) -> IcalLeaf<'static> {
    IcalLeaf::from(match bytes {
        b"\r\n" => "\r\n",
        b"\n" => "\n",
        _ => "",
    })
}

/// Whether a line ending is a `\r\n` rather than a bare `\n`.
fn is_crlf(eol: &[u8]) -> bool {
    eol.starts_with(b"\r")
}

/// Bytes the tokeniser resolved away, as text. Every one of them is a line
/// break, a space, a tab or an `=`, so the conversion never fails; a lone `""`
/// on the impossible path keeps this total rather than panicking.
fn ascii(bytes: &[u8]) -> &str {
    text(bytes).unwrap_or("")
}

/// A lossy owned string of raw bytes, for error diagnostics.
fn lossy(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString;

    use crate::tree::{line::IcalLine, value::node::IcalValueNode};

    #[test]
    fn reads_the_whole_raw_value_past_its_separators() {
        // NOTE: Reading the first `;`-component alone turned this rule into
        // an endless weekly one.
        let (mut line, _) = IcalLine::take(b"RRULE:FREQ=WEEKLY;COUNT=3;BYDAY=MO,WE\r\n").unwrap();
        assert_eq!(line.raw_value_str(), "FREQ=WEEKLY;COUNT=3;BYDAY=MO,WE");

        // NOTE: Split by an edit, the value still reads whole, escapes kept.
        line.value.set(&["a,b;c"]);
        assert_eq!(line.raw_value_str(), r"a\,b\;c");
        assert_eq!(&*line.raw_value(), br"a\,b\;c");
    }

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
    fn keeps_a_quoted_parameter_value_whole() {
        // NOTE: RFC 5545 section 3.2 lets a quoted parameter value carry a
        // colon; section 3.2.1 uses one in its ALTREP example.
        let raw = "DESCRIPTION;ALTREP=\"cid:part1.0001@example.org\";LANGUAGE=en:Meeting notes\r\n";
        let (line, _) = IcalLine::take(raw.as_bytes()).unwrap();

        assert_eq!(line.name.get(), "DESCRIPTION");
        assert_eq!(line.params.len(), 2);
        assert_eq!(line.params[0].name.get(), "ALTREP");
        assert_eq!(
            line.params[0].values[0].get(),
            "\"cid:part1.0001@example.org\""
        );
        assert_eq!(line.params[1].name.get(), "LANGUAGE");
        assert_eq!(line.raw_value_str(), "Meeting notes");
        assert_eq!(line.to_string(), raw);
    }

    #[test]
    fn keeps_a_quoted_semicolon_out_of_the_parameter_split() {
        let raw = "ATTENDEE;DIR=\"ldap://host:389/cn=Ada;o=Example\":mailto:ada@example.com\r\n";
        let (line, _) = IcalLine::take(raw.as_bytes()).unwrap();

        assert_eq!(line.params.len(), 1);
        assert_eq!(line.params[0].name.get(), "DIR");
        assert_eq!(line.raw_value_str(), "mailto:ada@example.com");
        assert_eq!(line.to_string(), raw);
    }

    #[test]
    fn an_unbalanced_quote_still_parses() {
        // NOTE: quote tracking alone would swallow the rest of the line, so
        // with no colon outside quotes the scan falls back to the first one.
        let raw = "ATTENDEE;CN=\"Ada:mailto:ada@example.com\r\n";
        let (line, _) = IcalLine::take(raw.as_bytes()).unwrap();

        assert_eq!(line.name.get(), "ATTENDEE");
        assert_eq!(line.to_string(), raw);
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
        let (mut line, _) = IcalLine::take(b"NOTE:foo\r\n bar\r\n").unwrap();
        line.value = IcalValueNode::parse(b"BARFOO");
        assert_eq!(line.to_string(), "NOTE:BAR\r\n FOO\r\n");
    }

    #[test]
    fn keeps_whitespace_beyond_the_single_fold_indicator() {
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
        assert_eq!(&*line.raw_value(), b"caf=C3=A9");
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
