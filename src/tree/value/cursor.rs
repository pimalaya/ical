//! # Value cursor
//!
//! The in-place edit cursor every property lens uses.
//!
//! A cursor borrows a content line mutably and reads and writes its value
//! through the codec: getters decode (unescape), setters encode (escape) and
//! write through to the syntax node.
//!
//! A setter only rewrites the component it touches, so every other leaf (and
//! every parameter) of a parsed line stays byte for byte intact.
//!
//! [`IcalValueCursor`] offers convenience accessors for the common
//! single-value and list shapes, plus component-level access for the
//! structured values (`GEO`, `REQUEST-STATUS`).
//!
//! Beside the UTF-8 text accessors it offers a raw byte hatch
//! ([`bytes`](IcalValueCursor::bytes) /
//! [`set_bytes`](IcalValueCursor::set_bytes)) for a value in a foreign
//! charset.
//!
//! Behind the content-encoding features sit the
//! [`quoted_printable`](IcalValueCursor::quoted_printable) and
//! [`charset`](IcalValueCursor::charset) decoders.

#[cfg(feature = "encoding")]
use alloc::string::String;
use alloc::{borrow::Cow, vec::Vec};

use crate::tree::{line::IcalLine, param::lens::IcalParamLens};

/// A typed cursor over a content line's value, editing in place and byte
/// preserving for the components it does not touch.
pub struct IcalValueCursor<'c, 'a> {
    /// The borrowed content line.
    pub line: &'c mut IcalLine<'a>,
}

impl IcalValueCursor<'_, '_> {
    /// The whole value as a single decoded text, its `;` and `,` kept literal.
    pub fn text(&self) -> Cow<'_, str> {
        self.line.value.decode()
    }

    /// Set the whole value to a single text, escaping it. Writes UTF-8; to keep
    /// a foreign charset, transcode yourself and use
    /// [`set_bytes`](Self::set_bytes).
    pub fn set_text(&mut self, value: impl AsRef<str>) {
        self.line.value.set(&[value]);
    }

    /// The whole value's raw bytes, unescaped but not otherwise decoded.
    ///
    /// Neither transcoded nor transfer-decoded, for a value carrying a foreign
    /// charset. To resolve `QUOTED-PRINTABLE` or a `CHARSET`, use the
    /// [`quoted_printable`](Self::quoted_printable) /
    /// [`charset`](Self::charset) feature helpers.
    pub fn bytes(&self) -> Cow<'_, [u8]> {
        self.line.value.decode_bytes()
    }

    /// Set the whole value to raw bytes (the foreign-charset escape hatch),
    /// escaping structural separators but writing the bytes verbatim. The
    /// calendar's `CHARSET` parameter is left untouched: it is the caller's to
    /// keep consistent.
    pub fn set_bytes(&mut self, value: impl AsRef<[u8]>) {
        self.line.value.set_bytes(&[value]);
    }

    /// Decode the value's `QUOTED-PRINTABLE` `=XX` octets to raw bytes.
    ///
    /// Only when the line declares that encoding, else the raw
    /// [`bytes`](Self::bytes). Still in the value's own (possibly foreign)
    /// charset; pair with [`charset`](Self::charset) to get text. Requires the
    /// `quoted-printable` feature.
    #[cfg(feature = "quoted-printable")]
    pub fn quoted_printable(&self) -> Vec<u8> {
        let raw = self.bytes();

        if self.line.is_quoted_printable() {
            quoted_printable::decode(raw.as_ref(), quoted_printable::ParseMode::Robust)
                .unwrap_or_else(|_| raw.into_owned())
        } else {
            raw.into_owned()
        }
    }

    /// Transcode the value to text using its `CHARSET` parameter (defaulting to
    /// UTF-8 when absent or unrecognised). When the `quoted-printable` feature
    /// is also on, `QUOTED-PRINTABLE` octets are resolved first. Requires the
    /// `encoding` feature.
    #[cfg(feature = "encoding")]
    pub fn charset(&self) -> String {
        #[cfg(feature = "quoted-printable")]
        let bytes = self.quoted_printable();
        #[cfg(not(feature = "quoted-printable"))]
        let bytes = self.bytes().into_owned();

        let encoding = self
            .line
            .charset_label()
            .and_then(|label| encoding_rs::Encoding::for_label(label.as_bytes()))
            .unwrap_or(encoding_rs::UTF_8);

        encoding.decode_without_bom_handling(&bytes).0.into_owned()
    }

    /// The whole value as a decoded list (its `,`-separated values), its `;`
    /// kept literal.
    pub fn list(&self) -> Vec<Cow<'_, str>> {
        self.line.value.decode_list()
    }

    /// Set the whole value to a list, escaping each value.
    pub fn set_list<S: AsRef<str>>(&mut self, values: &[S]) {
        self.line.value.set(values);
    }

    /// The `i`th component as a decoded list, for structured values.
    pub fn component(&self, i: usize) -> Vec<Cow<'_, str>> {
        self.line.value.decode_component_list(i)
    }

    /// Set the `i`th component, escaping each value and preserving the rest.
    pub fn set_component<S: AsRef<str>>(&mut self, i: usize, values: &[S]) {
        self.line.value.set_component(i, values);
    }

    /// The first parameter of type `P` on this line, decoded.
    pub fn param<P: IcalParamLens>(&self) -> Option<P::Target<'_>> {
        self.line.param::<P>()
    }
}

#[cfg(test)]
mod tests {
    use alloc::{
        format,
        string::{String, ToString},
        vec,
    };

    use crate::{prop::summary::SUMMARY, tree::cst::IcalCst};

    const HEAD: &str = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//x//EN\r\n";
    const TAIL: &str = "END:VCALENDAR\r\n";

    fn cal(prop_line: &str) -> String {
        format!("{HEAD}{prop_line}\r\n{TAIL}")
    }

    #[test]
    fn edits_a_scalar_value_in_place_escaping_it() {
        let raw = cal("SUMMARY:Lunch");
        let mut c = IcalCst::parse(&raw).unwrap();
        c.prop_mut::<SUMMARY>().unwrap().set_text("Tea, now");
        assert!(c.to_string().contains("SUMMARY:Tea\\, now\r\n"));
    }

    #[test]
    fn keeps_a_newline_written_into_a_vcalendar_1_0_value_on_its_line() {
        // NOTE: Versit has no newline escape, and the raw byte would end the
        // line, leaving a calendar the parser refuses.
        let raw = "BEGIN:VCALENDAR\r\nVERSION:1.0\r\nSUMMARY:x\r\nEND:VCALENDAR\r\n";
        let mut c = IcalCst::parse(raw.as_bytes()).unwrap();
        c.prop_mut::<SUMMARY>().unwrap().set_text("a\nb");

        assert!(c.to_string().contains("SUMMARY:a\\nb\r\n"));
        assert!(IcalCst::parse(&c.to_bytes()).is_ok());
    }

    #[test]
    fn writes_and_reads_a_foreign_charset_value_as_raw_bytes() {
        use crate::prop::description::DESCRIPTION;

        let raw = cal("DESCRIPTION;CHARSET=ISO-8859-1:x");
        let mut c = IcalCst::parse(&raw).unwrap();

        // NOTE: "café" in ISO-8859-1: the trailing 0xE9 is not valid UTF-8.
        let latin1 = [b'c', b'a', b'f', 0xE9];
        c.prop_mut::<DESCRIPTION>().unwrap().set_bytes(latin1);

        assert_eq!(
            c.prop_mut::<DESCRIPTION>().unwrap().bytes().as_ref(),
            &latin1,
        );
        assert!(c.to_bytes().windows(4).any(|window| window == latin1));
    }

    #[cfg(feature = "quoted-printable")]
    #[test]
    fn quoted_printable_helper_resolves_octets() {
        use crate::prop::description::DESCRIPTION;

        let raw = cal("DESCRIPTION;CHARSET=ISO-8859-1;ENCODING=QUOTED-PRINTABLE:caf=E9");
        let mut c = IcalCst::parse(&raw).unwrap();

        assert_eq!(
            c.prop_mut::<DESCRIPTION>().unwrap().quoted_printable(),
            [b'c', b'a', b'f', 0xE9],
        );
    }

    #[cfg(all(feature = "encoding", feature = "quoted-printable"))]
    #[test]
    fn charset_helper_transcodes_to_utf8() {
        use crate::prop::description::DESCRIPTION;

        let raw = cal("DESCRIPTION;CHARSET=ISO-8859-1;ENCODING=QUOTED-PRINTABLE:caf=E9");
        let mut c = IcalCst::parse(&raw).unwrap();

        assert_eq!(c.prop_mut::<DESCRIPTION>().unwrap().charset(), "café");
    }

    #[test]
    fn edits_one_structured_component_preserving_the_rest() {
        use crate::prop::geo::GEO;

        let raw = cal("GEO:37.0;-122.0");
        let mut c = IcalCst::parse(&raw).unwrap();
        c.prop_mut::<GEO>().unwrap().set_component(1, &["-100.0"]);
        assert!(c.to_string().contains("GEO:37.0;-100.0\r\n"));
    }

    /// The generic accessors read and write the value, not its first slot.
    ///
    /// A semicolon separates nothing in a text value, so a read that stopped
    /// at one handed back a truncated value and a write that rewrote only the
    /// first component left the rest behind: read then write changed it.
    #[test]
    fn reads_and_writes_the_whole_value_not_its_first_component() {
        use crate::prop::description::DESCRIPTION;

        let raw = cal("DESCRIPTION:a;b");
        let mut c = IcalCst::parse(&raw).unwrap();

        {
            let cursor = c.prop_mut::<DESCRIPTION>().unwrap();
            assert_eq!(cursor.text(), "a;b");
            assert_eq!(cursor.bytes().as_ref(), b"a;b");
            assert_eq!(cursor.list(), vec!["a;b"]);
        }

        let whole = c.prop_mut::<DESCRIPTION>().unwrap().text().into_owned();
        c.prop_mut::<DESCRIPTION>().unwrap().set_text(&whole);

        assert!(c.to_string().contains("DESCRIPTION:a\\;b\r\n"), "got: {c}");
        assert_eq!(c.prop_mut::<DESCRIPTION>().unwrap().text(), "a;b");
    }

    /// A structured value read through its lens keeps its components' commas.
    #[test]
    fn reads_a_structured_component_past_its_first_comma() {
        use crate::prop::request_status::REQUEST_STATUS;

        let raw = cal("REQUEST-STATUS:2.0;ok;rcpt,two");
        let c = IcalCst::parse(&raw).unwrap();
        let status = c.prop::<REQUEST_STATUS>().unwrap();

        assert_eq!(status.description, "ok");
        assert_eq!(status.extra, "rcpt,two");
    }

    #[test]
    fn exercises_every_generic_accessor() {
        use crate::prop::categories::CATEGORIES;

        let raw = cal("CATEGORIES:a,b");
        let mut c = IcalCst::parse(&raw).unwrap();

        {
            let mut cursor = c.prop_mut::<CATEGORIES>().unwrap();

            // NOTE: A text read takes the whole value and a list read splits it
            // on its commas, both keeping every `;` the value carries, while a
            // component read takes one `;`-separated slot.
            assert_eq!(cursor.text(), "a,b");
            assert_eq!(cursor.list(), vec!["a", "b"]);
            assert_eq!(cursor.component(0), vec!["a", "b"]);

            cursor.set_text("x");
            assert_eq!(cursor.text(), "x");

            cursor.set_list(&["a", "b"]);
            assert_eq!(cursor.list(), vec!["a", "b"]);

            // A component past the last one extends the value rather than
            // dropping the write.
            cursor.set_component(1, &["y"]);
            assert_eq!(cursor.component(1), vec!["y"]);
        }

        assert!(c.to_string().contains("CATEGORIES:a,b;y\r\n"), "got: {c}");
    }
}
