//! # LINK
//!
//! The `LINK` property: a typed link from the component to a related resource
//! (RFC 9253 6.1).

use crate::{
    param::IcalParamKind,
    prop::{IcalPropKind, spec::IcalPropSpec},
    value::IcalValueKind,
    version::IcalVersion,
};

/// The `LINK` property marker.
pub struct LINK;

impl IcalPropSpec for LINK {
    const KIND: IcalPropKind = IcalPropKind::Link;

    fn allowed_versions() -> &'static [IcalVersion] {
        &[IcalVersion::V2_0]
    }

    fn allowed_values(_version: IcalVersion) -> &'static [IcalValueKind] {
        &[IcalValueKind::Uri, IcalValueKind::Text]
    }

    /// RFC 9253 8.1: a link states what it links to with `LINKREL`.
    fn allowed_params(_version: IcalVersion) -> &'static [IcalParamKind] {
        &[
            IcalParamKind::Value,
            IcalParamKind::LinkRel,
            IcalParamKind::Label,
            IcalParamKind::FmtType,
        ]
    }
}
