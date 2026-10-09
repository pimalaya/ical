//! # STYLED-DESCRIPTION
//!
//! The `STYLED-DESCRIPTION` property: the description in a richer text format
//! than `DESCRIPTION` (RFC 9073 6.5).

use crate::{
    param::IcalParamKind,
    prop::{IcalPropKind, spec::IcalPropSpec},
    value::IcalValueKind,
    version::IcalVersion,
};

/// The `STYLED-DESCRIPTION` property marker.
#[allow(non_camel_case_types)]
pub struct STYLED_DESCRIPTION;

impl IcalPropSpec for STYLED_DESCRIPTION {
    const KIND: IcalPropKind = IcalPropKind::StyledDescription;

    fn allowed_versions() -> &'static [IcalVersion] {
        &[IcalVersion::V2_0]
    }

    /// Inline text, or a `URI` to fetch it from.
    fn allowed_values(_version: IcalVersion) -> &'static [IcalValueKind] {
        &[IcalValueKind::Text, IcalValueKind::Uri]
    }

    /// The text's media type and whether it was derived from another
    /// description (RFC 9073 5.3), beside the parameters of `DESCRIPTION`.
    fn allowed_params(_version: IcalVersion) -> &'static [IcalParamKind] {
        &[
            IcalParamKind::Value,
            IcalParamKind::AltRep,
            IcalParamKind::Language,
            IcalParamKind::FmtType,
            IcalParamKind::Derived,
        ]
    }
}
