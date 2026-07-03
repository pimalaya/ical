//! # Value cursor
//!
//! The generic in-place edit cursor used by every property lens but `N`.
//!
//! A cursor borrows a content line mutably and lets you read and write its
//! value through the codec: getters decode (unescape), setters encode (escape)
//! and write through to the syntax node. Crucially, a setter only rewrites the
//! component it touches, so every other leaf (and every parameter) of a parsed
//! line stays byte for byte intact. [`IcalValueCursor`] exposes both
//! convenience accessors for the common single-value and list shapes and raw
//! component-level access for the structured kinds (`ADR`, `GENDER`, `ORG`,
//! `CLIENTPIDMAP`); the bespoke
//! [`IcalNCursor`](crate::tree::prop::n::IcalNCursor) names `N`'s components.
//!
//! Beside the UTF-8 text accessors it offers a raw byte hatch
//! ([`bytes`](IcalValueCursor::bytes) /
//! [`set_bytes`](IcalValueCursor::set_bytes)) for a value in a foreign
//! charset, and, behind the content-encoding features, the
//! [`quoted_printable`](IcalValueCursor::quoted_printable) and
//! [`charset`](IcalValueCursor::charset) decoders.

use alloc::{borrow::Cow, vec::Vec};

use crate::tree::{line::IcalLine, param::IcalParamLens};

/// A typed cursor over a content line's value, editing in place and byte
/// preserving for the components it does not touch.
pub struct IcalValueCursor<'c, 'a> {
    /// The borrowed content line.
    pub line: &'c mut IcalLine<'a>,
}

impl IcalValueCursor<'_, '_> {
    /// The whole value as a single decoded text (component 0, value 0).
    pub fn text(&self) -> Cow<'_, str> {
        self.line.value.decode_scalar_at(0)
    }

    /// Set the value to a single text, escaping and preserving any other
    /// components. Writes UTF-8; to keep a foreign charset, transcode yourself
    /// and use [`set_bytes`](Self::set_bytes).
    pub fn set_text(&mut self, value: impl AsRef<str>) {
        self.line.value.set_at(0, &[value]);
    }

    /// The whole value's raw bytes (component 0, value 0), unescaped but not
    /// transcoded and not transfer-decoded, for a value carrying a foreign
    /// charset. To resolve `QUOTED-PRINTABLE` or a `CHARSET`, use the
    /// [`quoted_printable`](Self::quoted_printable) /
    /// [`charset`](Self::charset) feature helpers.
    pub fn bytes(&self) -> Cow<'_, [u8]> {
        self.line.value.decode_bytes_at(0)
    }

    /// Set the value to raw bytes (the foreign-charset escape hatch), escaping
    /// structural separators but writing the bytes verbatim and preserving any
    /// other components. The card's `CHARSET` parameter is left untouched: it
    /// is the caller's to keep consistent.
    pub fn set_bytes(&mut self, value: impl AsRef<[u8]>) {
        self.line.value.set_bytes_at(0, &[value]);
    }

    /// Decode the value's `QUOTED-PRINTABLE` `=XX` octets to raw bytes when the
    /// line declares that encoding, else the raw [`bytes`](Self::bytes). Still
    /// in the value's own (possibly foreign) charset; pair with
    /// [`charset`](Self::charset) to get text. Requires the `quoted-printable`
    /// feature.
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
    pub fn charset(&self) -> alloc::string::String {
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

    /// The value's first component as a decoded list (its `,`-separated
    /// values).
    pub fn list(&self) -> Vec<Cow<'_, str>> {
        self.line.value.decode_at(0)
    }

    /// Set the value's first component to a list, escaping each value.
    pub fn set_list<S: AsRef<str>>(&mut self, values: &[S]) {
        self.line.value.set_at(0, values);
    }

    /// The `i`th component as a decoded list, for structured values.
    pub fn component(&self, i: usize) -> Vec<Cow<'_, str>> {
        self.line.value.decode_at(i)
    }

    /// Set the `i`th component, escaping each value and preserving the rest.
    pub fn set_component<S: AsRef<str>>(&mut self, i: usize, values: &[S]) {
        self.line.value.set_at(i, values);
    }

    /// The first parameter of type `P` on this line, decoded.
    pub fn param<P: IcalParamLens>(&self) -> Option<P::Target<'_>> {
        self.line.param::<P>()
    }
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString;

    use crate::tree::{cst::IcalCst, prop::summary::SUMMARY};

    const HEAD: &str = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//x//EN\r\n";
    const TAIL: &str = "END:VCALENDAR\r\n";

    fn cal(prop_line: &str) -> alloc::string::String {
        alloc::format!("{HEAD}{prop_line}\r\n{TAIL}")
    }

    #[test]
    fn edits_a_scalar_value_in_place_escaping_it() {
        let raw = cal("SUMMARY:Lunch");
        let mut c = IcalCst::parse(&raw).unwrap();
        c.prop_mut::<SUMMARY>().unwrap().set_text("Tea, now");
        assert!(c.to_string().contains("SUMMARY:Tea\\, now\r\n"));
    }

    #[test]
    fn writes_and_reads_a_foreign_charset_value_as_raw_bytes() {
        use crate::tree::prop::description::DESCRIPTION;

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
        use crate::tree::prop::description::DESCRIPTION;

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
        use crate::tree::prop::description::DESCRIPTION;

        let raw = cal("DESCRIPTION;CHARSET=ISO-8859-1;ENCODING=QUOTED-PRINTABLE:caf=E9");
        let mut c = IcalCst::parse(&raw).unwrap();

        assert_eq!(c.prop_mut::<DESCRIPTION>().unwrap().charset(), "café");
    }

    #[test]
    fn edits_one_structured_component_preserving_the_rest() {
        use crate::tree::prop::geo::GEO;

        let raw = cal("GEO:37.0;-122.0");
        let mut c = IcalCst::parse(&raw).unwrap();
        c.prop_mut::<GEO>().unwrap().set_component(1, &["-100.0"]);
        assert!(c.to_string().contains("GEO:37.0;-100.0\r\n"));
    }

    #[test]
    fn exercises_every_generic_accessor() {
        use crate::tree::prop::categories::CATEGORIES;

        let raw = cal("CATEGORIES:a,b");
        let mut c = IcalCst::parse(&raw).unwrap();
        let mut cursor = c.prop_mut::<CATEGORIES>().unwrap();

        let _ = cursor.text();
        let _ = cursor.list();
        let _ = cursor.component(0);
        cursor.set_text("x");
        cursor.set_list(&["a", "b"]);
        cursor.set_component(1, &["y"]);
    }
}
