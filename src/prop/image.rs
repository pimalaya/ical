//! # IMAGE
//!
//! The `IMAGE` property: an image for the calendar or component, by URI or
//! inline (RFC 7986 5.10).

use crate::{
    param::IcalParamKind,
    prop::{IcalPropKind, spec::IcalPropSpec},
    value::IcalValueKind,
    version::IcalVersion,
};

/// The `IMAGE` property marker.
pub struct IMAGE;

impl IcalPropSpec for IMAGE {
    const KIND: IcalPropKind = IcalPropKind::Image;

    fn allowed_versions() -> &'static [IcalVersion] {
        &[IcalVersion::V2_0]
    }

    /// An inline image is a `BINARY` beside the default `URI`.
    fn allowed_values(_version: IcalVersion) -> &'static [IcalValueKind] {
        &[IcalValueKind::Uri, IcalValueKind::Binary]
    }

    /// A media type, the `ENCODING=BASE64` an inline image requires, an
    /// alternate representation and how to display it (RFC 7986 6.1).
    fn allowed_params(_version: IcalVersion) -> &'static [IcalParamKind] {
        &[
            IcalParamKind::Value,
            IcalParamKind::FmtType,
            IcalParamKind::Encoding,
            IcalParamKind::AltRep,
            IcalParamKind::Display,
        ]
    }
}
