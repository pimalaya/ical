//! # STRUCTURED-DATA
//!
//! The `STRUCTURED-DATA` property: machine-readable data about the component,
//! inline or by URI (RFC 9073 6.6).

use crate::{
    param::IcalParamKind,
    prop::{IcalPropKind, spec::IcalPropSpec},
    value::IcalValueKind,
    version::IcalVersion,
};

/// The `STRUCTURED-DATA` property marker.
#[allow(non_camel_case_types)]
pub struct STRUCTURED_DATA;

impl IcalPropSpec for STRUCTURED_DATA {
    const KIND: IcalPropKind = IcalPropKind::StructuredData;

    fn allowed_versions() -> &'static [IcalVersion] {
        &[IcalVersion::V2_0]
    }

    /// Inline text, an inline `BINARY` body, or a `URI` to fetch it from.
    fn allowed_values(_version: IcalVersion) -> &'static [IcalValueKind] {
        &[
            IcalValueKind::Text,
            IcalValueKind::Binary,
            IcalValueKind::Uri,
        ]
    }

    /// The data's media type and schema (RFC 9073 5.2), and the
    /// `ENCODING=BASE64` an inline body requires.
    fn allowed_params(_version: IcalVersion) -> &'static [IcalParamKind] {
        &[
            IcalParamKind::Value,
            IcalParamKind::FmtType,
            IcalParamKind::Schema,
            IcalParamKind::Encoding,
        ]
    }
}
