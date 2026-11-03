//! # Wire shape
//!
//! What a content line looked like on the wire, kept beside the logical line it
//! parsed into.
//!
//! A real calendar folds at 75 octets, blank-lines its components apart and,
//! under vCalendar 1.0, breaks a `QUOTED-PRINTABLE` value across physical
//! lines.
//!
//! Every layer above the parser wants the *logical* line, so
//! [`IcalLine::take`](crate::tree::line::IcalLine::take) resolves all three
//! away.
//!
//! [`IcalWire`] is what makes that resolution reversible: a list of byte
//! offsets into the logical line, each holding the bytes the wire carried
//! there, so serialization reproduces the input exactly rather than a
//! normalised paraphrase of it.
//!
//! ## Offsets are logical, and checked
//!
//! An offset indexes the line's logical bytes (its name, its parameters and
//! its value, exactly as `IcalLine::write_bytes` lays them out, line ending
//! excluded, which is why a blank line before the line is an insertion at
//! offset 0).
//!
//! The logical length is recorded with them, and a shape whose length no
//! longer matches is dropped rather than applied: an edit that changes a
//! value's length moves every byte after it, so the old fold points would
//! land in the wrong places.
//!
//! A line no shape was recorded against, built rather than parsed or edited
//! out of the shape it had, is folded at 75 octets instead, as RFC 5545 3.1
//! asks, never inside a UTF-8 sequence.

use alloc::{borrow::Cow, vec::Vec};

/// The longest a physical line runs, its line ending excluded (RFC 5545 3.1).
pub(crate) const FOLD_OCTETS: usize = 75;

/// One piece of wire the parser resolved away.
#[derive(Clone, Debug)]
pub enum IcalWirePart<'a> {
    /// An RFC 5545 3.1 fold: a line break, then the single whitespace that
    /// marked the continuation.
    Fold {
        /// Whether the break was `\r\n` rather than a bare `\n`.
        crlf: bool,
        /// The folding whitespace, a space or a tab.
        wsp: u8,
    },
    /// A `QUOTED-PRINTABLE` soft line break: an `=` and the break after it.
    Soft {
        /// Whether the break was `\r\n` rather than a bare `\n`.
        crlf: bool,
    },
    /// Bytes taken verbatim off the wire and dropped: the blank lines before a
    /// content line, the whitespace of a dangling continuation, or a trailing
    /// `=` left over from a soft break with nothing to continue.
    Skipped(Cow<'a, str>),
}

impl IcalWirePart<'_> {
    /// Write the piece back out.
    fn write_bytes(&self, out: &mut Vec<u8>) {
        match self {
            Self::Fold { crlf, wsp } => {
                write_eol(*crlf, out);
                out.push(*wsp);
            }
            Self::Soft { crlf } => {
                out.push(b'=');
                write_eol(*crlf, out);
            }
            Self::Skipped(bytes) => out.extend_from_slice(bytes.as_bytes()),
        }
    }

    /// Convert into an owned piece (`'static`).
    fn into_static(self) -> IcalWirePart<'static> {
        match self {
            Self::Fold { crlf, wsp } => IcalWirePart::Fold { crlf, wsp },
            Self::Soft { crlf } => IcalWirePart::Soft { crlf },
            Self::Skipped(bytes) => IcalWirePart::Skipped(Cow::Owned(bytes.into_owned())),
        }
    }
}

fn write_eol(crlf: bool, out: &mut Vec<u8>) {
    out.extend_from_slice(if crlf { b"\r\n" } else { b"\n" });
}

/// Fold the logical line written in `out` from `start` on at [`FOLD_OCTETS`]
/// (RFC 5545 3.1), in place: a line break and a space before every
/// continuation, the space counting toward its length.
///
/// A fold never lands inside a UTF-8 sequence: the cut backs off over the
/// continuation bytes, at most three, so bytes that are not UTF-8 still go out.
pub(crate) fn fold(out: &mut Vec<u8>, start: usize, crlf: bool) {
    let len = out.len() - start;
    let mut at = 0;
    let mut room = FOLD_OCTETS;

    while len - at > room {
        let mut cut = at + room;
        let floor = (cut - 3).max(at + 1);

        while cut > floor && out[start + cut] & 0xC0 == 0x80 {
            cut -= 1;
        }

        out.extend_from_within(start + at..start + cut);
        write_eol(crlf, out);
        out.push(b' ');

        at = cut;
        room = FOLD_OCTETS - 1;
    }

    out.extend_from_within(start + at..start + len);
    out.copy_within(start + len.., start);
    out.truncate(out.len() - len);
}

/// The wire shape of one content line: every piece the parser resolved away,
/// with the offset it sat at and the logical length those offsets index.
///
/// Never sealed for a line that was built rather than parsed, and empty for a
/// line whose wire shape *is* its logical shape (unfolded, with no blank line
/// before it).
#[derive(Clone, Debug, Default)]
pub struct IcalWire<'a> {
    /// The pieces, in the order they occur on the wire.
    parts: Vec<(usize, IcalWirePart<'a>)>,
    /// The logical length these offsets were taken against, none for a line
    /// that was built rather than parsed.
    len: Option<usize>,
}

impl<'a> IcalWire<'a> {
    /// The shape of a line built rather than parsed.
    pub(crate) const fn new() -> Self {
        Self {
            parts: Vec::new(),
            len: None,
        }
    }

    /// Whether the shape records no piece.
    pub fn is_empty(&self) -> bool {
        self.parts.is_empty()
    }

    /// About how many bytes the line takes on the wire, its ending aside:
    /// exact for an unedited parsed line folded with `\r\n`, a guess to size
    /// a buffer by otherwise.
    pub(crate) fn size_hint(&self) -> usize {
        self.len.unwrap_or(0) + 3 * self.parts.len()
    }

    /// The folds recorded, each as its offset and whether the break before it
    /// was a `\r\n`.
    pub(crate) fn folds(&self) -> impl Iterator<Item = (usize, bool)> + '_ {
        self.parts.iter().filter_map(|(offset, part)| match part {
            IcalWirePart::Fold { crlf, .. } => Some((*offset, *crlf)),
            _ => None,
        })
    }

    /// Record a fold at `offset`.
    pub(crate) fn fold(&mut self, offset: usize, crlf: bool, wsp: u8) {
        self.parts.push((offset, IcalWirePart::Fold { crlf, wsp }));
    }

    /// Record a `QUOTED-PRINTABLE` soft break at `offset`.
    pub(crate) fn soft(&mut self, offset: usize, crlf: bool) {
        self.parts.push((offset, IcalWirePart::Soft { crlf }));
    }

    /// Record bytes dropped verbatim at `offset`.
    pub(crate) fn skipped(&mut self, offset: usize, bytes: &'a str) {
        self.parts
            .push((offset, IcalWirePart::Skipped(Cow::Borrowed(bytes))));
    }

    /// Record `count` dropped `=` at `offset`, after every piece recorded at
    /// or before it, the pieces being in offset order.
    ///
    /// The dangling `=` of a `QUOTED-PRINTABLE` value, which the head splitter
    /// finds once the tokeniser recorded the rest: a soft break at the same
    /// offset or past it would otherwise go out first and swallow a line.
    pub(crate) fn insert_skipped_equals(&mut self, offset: usize, count: usize) {
        const EQUALS: &str = "================================================================";

        let bytes = match EQUALS.get(..count) {
            Some(bytes) => Cow::Borrowed(bytes),
            None => Cow::Owned("=".repeat(count)),
        };

        let at = self.parts.partition_point(|(at, _)| *at <= offset);
        self.parts
            .insert(at, (offset, IcalWirePart::Skipped(bytes)));
    }

    /// Pin the logical length the offsets were taken against.
    pub(crate) fn seal(&mut self, len: usize) {
        self.len = Some(len);
    }

    /// Whether the shape was recorded against a logical line of `len` bytes,
    /// so its offsets still index them.
    pub(crate) fn fits(&self, len: usize) -> bool {
        self.len == Some(len)
    }

    /// Put `earlier`'s pieces before this shape's, keeping the sealed length.
    ///
    /// The tokeniser records what it resolved (blank lines, folds, soft breaks)
    /// and the line splitter records a dangling `=` the value ends on. The two
    /// lists are each ordered, and a piece sitting at the same offset in both
    /// belongs to the tokeniser first, so a stable sort by offset merges them.
    ///
    /// The sort is not cosmetic. A value ending on two `=` gives the tokeniser
    /// a soft break past the last logical byte and the splitter a dangling `=`
    /// before it, so concatenating alone would emit the soft break first and
    /// the reparsed line would swallow the one that follows.
    pub(crate) fn prepend(&mut self, mut earlier: IcalWire<'a>) {
        if earlier.parts.is_empty() {
            return;
        }

        if self.parts.is_empty() {
            self.parts = earlier.parts;
            return;
        }

        earlier.parts.append(&mut self.parts);
        earlier.parts.sort_by_key(|(offset, _)| *offset);
        self.parts = earlier.parts;
    }

    /// Lay the logical line written in `out` from `start` on back out to the
    /// wire, re-inserting every piece, in place.
    ///
    /// The wire bytes are written past the logical ones, then moved down over
    /// them, so no buffer is allocated for a folded line.
    ///
    /// A shape that no longer [`fits`](Self::fits) the line is stale, left by
    /// an edit, and is not applied.
    pub(crate) fn lay_out(&self, out: &mut Vec<u8>, start: usize) {
        let len = out.len() - start;

        if self.parts.is_empty() || !self.fits(len) {
            return;
        }

        let mut at = 0;

        for (offset, part) in &self.parts {
            // NOTE: Clamped, so a shape recorded against other bytes can never
            // index out of this line or walk backwards.
            let offset = (*offset).clamp(at, len);
            out.extend_from_within(start + at..start + offset);
            part.write_bytes(out);
            at = offset;
        }

        out.extend_from_within(start + at..start + len);
        out.copy_within(start + len.., start);
        out.truncate(out.len() - len);
    }

    /// Lay a line out to the wire as [`lay_out`](Self::lay_out) does, its head
    /// (name, parameters and colon) written in `out` from `start` on and its
    /// value read from `value`, which is copied once.
    ///
    /// The pieces before the head go in front of it. A piece inside the head
    /// (a fold before the colon) takes the general way. The caller checked the
    /// shape [`fits`](Self::fits) the line.
    pub(crate) fn lay_out_value(&self, out: &mut Vec<u8>, start: usize, value: &[u8]) {
        if self.parts.is_empty() {
            return out.extend_from_slice(value);
        }

        let head = out.len() - start;
        let len = head + value.len();

        let lead = self.parts.iter().take_while(|(at, _)| *at == 0).count();
        let (front, back) = self.parts.split_at(lead);

        if back.iter().any(|(at, _)| *at < head) {
            out.extend_from_slice(value);
            return self.lay_out(out, start);
        }

        if !front.is_empty() {
            for (_, part) in front {
                part.write_bytes(out);
            }

            let pieces = out.len() - start - head;
            out[start..].rotate_right(pieces);
        }

        let mut at = head;

        for (offset, part) in back {
            // NOTE: Clamped as in `lay_out`.
            let offset = (*offset).clamp(at, len);
            out.extend_from_slice(&value[at - head..offset - head]);
            part.write_bytes(out);
            at = offset;
        }

        out.extend_from_slice(&value[at - head..]);
    }

    /// Convert into an owned shape (`'static`).
    pub(crate) fn into_static(self) -> IcalWire<'static> {
        IcalWire {
            parts: self
                .parts
                .into_iter()
                .map(|(offset, part)| (offset, part.into_static()))
                .collect(),
            len: self.len,
        }
    }
}

#[cfg(test)]
mod tests {
    use core::str;

    use alloc::{vec, vec::Vec};

    use crate::tree::wire::{IcalWire, fold};

    fn written(wire: &IcalWire<'_>, logical: &[u8]) -> Vec<u8> {
        let mut out = b"before".to_vec();
        out.extend_from_slice(logical);
        wire.lay_out(&mut out, 6);
        let line = out.split_off(6);
        assert_eq!(out, b"before");
        line
    }

    #[test]
    fn re_inserts_a_fold_where_it_was() {
        let mut wire = IcalWire::default();
        wire.fold(3, true, b' ');
        wire.seal(6);

        assert_eq!(written(&wire, b"foobar"), b"foo\r\n bar");
    }

    #[test]
    fn re_inserts_a_blank_line_before_the_name() {
        let mut wire = IcalWire::default();
        wire.skipped(0, "\r\n");
        wire.seal(6);

        assert_eq!(written(&wire, b"foobar"), b"\r\nfoobar");
    }

    #[test]
    fn re_inserts_a_soft_break() {
        let mut wire = IcalWire::default();
        wire.soft(3, false);
        wire.seal(6);

        assert_eq!(written(&wire, b"foobar"), b"foo=\nbar");
    }

    #[test]
    fn drops_a_shape_taken_against_other_bytes() {
        // NOTE: What an edit leaves behind: the value grew, so every fold point
        // after it is wrong and the whole shape has to go.
        let mut wire = IcalWire::default();
        wire.fold(3, true, b' ');
        wire.seal(6);

        assert_eq!(written(&wire, b"foobarbaz"), b"foobarbaz");
    }

    #[test]
    fn keeps_the_order_of_pieces_at_one_offset() {
        let mut wire = IcalWire::default();
        wire.skipped(0, "\r\n");
        wire.skipped(0, " ");
        wire.seal(3);

        assert_eq!(written(&wire, b"foo"), b"\r\n foo");
    }

    #[test]
    fn prepends_an_earlier_shape_before_a_later_one() {
        let mut earlier = IcalWire::default();
        earlier.skipped(0, "\r\n");

        let mut wire = IcalWire::default();
        wire.skipped(3, "=");
        wire.seal(3);
        wire.prepend(earlier);

        assert_eq!(written(&wire, b"foo"), b"\r\nfoo=");
    }

    #[test]
    fn orders_a_merged_shape_by_offset_rather_than_by_list() {
        // NOTE: What a value ending on two `=` leaves: a soft break past the
        // last logical byte, and the dangling `=` that precedes it.
        let mut earlier = IcalWire::default();
        earlier.soft(4, true);

        let mut wire = IcalWire::default();
        wire.skipped(3, "=");
        wire.seal(3);
        wire.prepend(earlier);

        assert_eq!(written(&wire, b"foo"), b"foo==\r\n");
    }

    fn folded(logical: &[u8]) -> Vec<u8> {
        let mut out = b"before".to_vec();
        out.extend_from_slice(logical);
        fold(&mut out, 6, true);
        let line = out.split_off(6);
        assert_eq!(out, b"before");
        line
    }

    /// The physical lines `folded` wrote, each with its break taken off.
    fn physical(out: &[u8]) -> Vec<&[u8]> {
        out.split(|&byte| byte == b'\n')
            .map(|line| line.strip_suffix(b"\r").unwrap_or(line))
            .collect()
    }

    #[test]
    fn folds_at_75_octets_counting_the_continuation_space() {
        let logical = vec![b'a'; 75 + 74 + 10];
        let out = folded(&logical);

        let lines = physical(&out);
        assert_eq!(
            lines.iter().map(|l| l.len()).collect::<Vec<_>>(),
            [75, 75, 11]
        );
        assert!(lines[1..].iter().all(|line| line[0] == b' '));
    }

    #[test]
    fn leaves_a_line_of_75_octets_whole() {
        let logical = vec![b'a'; 75];
        assert_eq!(folded(&logical), logical);
    }

    #[test]
    fn never_folds_inside_a_utf8_sequence() {
        // NOTE: A two-octet `é` and a four-octet emoji, each straddling octet
        // 75, the cut backing off to the octet each one starts at.
        for (fill, wide) in [(74, "é"), (73, "\u{1F600}")] {
            let mut logical = vec![b'a'; fill];
            logical.extend_from_slice(wide.as_bytes());
            logical.extend_from_slice(&[b'b'; 10]);

            let out = folded(&logical);
            let lines = physical(&out);

            assert_eq!(lines[0].len(), fill, "{wide}");
            assert!(lines.iter().all(|line| str::from_utf8(line).is_ok()));
        }
    }

    #[test]
    fn folds_bytes_that_are_not_utf8() {
        // NOTE: Continuation-looking octets all along, so no sequence start to
        // back off to: the cut still moves forward by at least 72 octets.
        let logical = vec![0x80; 200];
        let out = folded(&logical);

        let unfolded: Vec<u8> = physical(&out)
            .iter()
            .enumerate()
            .flat_map(|(i, line)| line[usize::from(i > 0)..].to_vec())
            .collect();

        assert_eq!(unfolded, logical);
        assert!(physical(&out).iter().all(|line| line.len() <= 75));
    }
}
