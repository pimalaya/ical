//! # Binary value
//!
//! The decoded binary value kind.
//!
//! Backs the binary-bearing properties (`ATTACH`, `IMAGE`, `STRUCTURED-DATA`)
//! where the value is inline base64 rather than an external URI reference (the
//! BINARY value, RFC 5545 3.3.1, carried with `ENCODING=BASE64`). When the same
//! property instead references an external URI it is decoded to
//! [`IcalUri`](crate::value::uri::IcalUri). The form is told by the line's
//! `VALUE` / `ENCODING` parameters; the payload is kept verbatim (base64 is not
//! decoded to bytes). Pure data; escaping lives entirely on the syntax side.

use alloc::borrow::Cow;

/// A decoded binary value: an external URI reference, or inline base64 kept as
/// its raw text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IcalBinary<'a> {
    /// An external URI reference.
    Uri(Cow<'a, str>),
    /// Inline base64 data, kept verbatim (not decoded to bytes).
    Base64(Cow<'a, str>),
}

#[cfg(feature = "base64")]
impl IcalBinary<'_> {
    /// Decode the inline [`Base64`](Self::Base64) payload to raw bytes; `None`
    /// for a [`Uri`](Self::Uri) reference, which embeds no data. Requires the
    /// `base64` feature.
    pub fn decode_base64(&self) -> Option<Result<alloc::vec::Vec<u8>, base64::DecodeError>> {
        use base64::prelude::{BASE64_STANDARD, Engine};

        match self {
            IcalBinary::Base64(data) => Some(BASE64_STANDARD.decode(data.as_bytes())),
            IcalBinary::Uri(_) => None,
        }
    }
}

#[cfg(all(test, feature = "base64"))]
mod tests {
    use alloc::borrow::Cow;

    use crate::value::binary::IcalBinary;

    #[test]
    fn decodes_inline_base64_but_not_a_uri() {
        let inline = IcalBinary::Base64(Cow::Borrowed("Zm9v"));
        assert_eq!(inline.decode_base64().unwrap().unwrap(), b"foo");

        let reference = IcalBinary::Uri(Cow::Borrowed("http://example.com/p.png"));
        assert!(reference.decode_base64().is_none());
    }
}
