//! # ATTACH
//!
//! The `ATTACH` property: a document associated with the component, a URI or an
//! inline `BASE64` body (RFC 5545 3.8.1.1).

use crate::{
    param::IcalParamKind,
    prop::{IcalPropKind, spec::IcalPropSpec},
    value::IcalValueKind,
    version::IcalVersion,
};

/// The `ATTACH` property marker.
pub struct ATTACH;

impl IcalPropSpec for ATTACH {
    const KIND: IcalPropKind = IcalPropKind::Attach;

    /// An inline body is a `BINARY` beside the default `URI`.
    fn allowed_values(_version: IcalVersion) -> &'static [IcalValueKind] {
        &[IcalValueKind::Uri, IcalValueKind::Binary]
    }

    /// A media type for either form, and the `ENCODING=BASE64` an inline body
    /// requires.
    fn allowed_params(_version: IcalVersion) -> &'static [IcalParamKind] {
        &[
            IcalParamKind::Value,
            IcalParamKind::FmtType,
            IcalParamKind::Encoding,
        ]
    }
}
