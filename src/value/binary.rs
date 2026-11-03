//! # Binary value
//!
//! The decoded binary value kind.
//!
//! Backs the binary-bearing properties (`ATTACH`, `IMAGE`, `STRUCTURED-DATA`)
//! where the value is inline base64 rather than an external URI reference
//! (the BINARY value, RFC 5545 3.3.1, carried with `ENCODING=BASE64`).
//!
//! A property referencing an external URI decodes to
//! [`IcalUri`](crate::value::uri::IcalUri) instead. The form is told by the
//! line's `VALUE` / `ENCODING` parameters, and the payload is kept verbatim
//! (base64 is not decoded to bytes).

use alloc::borrow::Cow;
#[cfg(feature = "base64")]
use alloc::vec::Vec;

#[cfg(feature = "base64")]
use base64_codec::codec::Base64Codec;

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
    /// Decode the inline [`Base64`](Self::Base64) payload to raw bytes, with
    /// whether it was well-formed; `None` for a [`Uri`](Self::Uri) reference,
    /// which embeds no data. Requires the `base64` feature.
    ///
    /// Decoding is liberal and runs the backend of the CPU token the
    /// application installed through simd-cpu: line breaks and blanks are
    /// skipped, any other byte outside the alphabet is skipped and reported,
    /// and missing padding is accepted.
    pub fn decode_base64(&self) -> Option<(Vec<u8>, bool)> {
        let IcalBinary::Base64(data) = self else {
            return None;
        };

        let mut bytes = Vec::new();
        let clean = Base64Codec::default().decode_vec(data.as_bytes(), &mut bytes);
        Some((bytes, clean))
    }
}

#[cfg(all(test, feature = "base64"))]
mod tests {
    use alloc::{borrow::Cow, vec::Vec};

    use crate::value::binary::IcalBinary;

    #[test]
    fn decodes_inline_base64_but_not_a_uri() {
        let inline = IcalBinary::Base64(Cow::Borrowed("Zm9v"));
        assert_eq!(inline.decode_base64(), Some((Vec::from(*b"foo"), true)));

        let dirty = IcalBinary::Base64(Cow::Borrowed("Zm9v*"));
        assert_eq!(dirty.decode_base64(), Some((Vec::from(*b"foo"), false)));

        let reference = IcalBinary::Uri(Cow::Borrowed("http://example.com/p.png"));
        assert!(reference.decode_base64().is_none());
    }
}
