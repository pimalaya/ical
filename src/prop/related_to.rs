//! # RELATED-TO
//!
//! The `RELATED-TO` property: the `UID` of a component this one relates to (RFC
//! 5545 3.8.4.5).

use crate::{
    param::IcalParamKind,
    prop::{IcalPropKind, spec::IcalPropSpec},
    version::IcalVersion,
};

/// The `RELATED-TO` property marker.
#[allow(non_camel_case_types)]
pub struct RELATED_TO;

impl IcalPropSpec for RELATED_TO {
    const KIND: IcalPropKind = IcalPropKind::RelatedTo;

    /// RFC 9253 6.2 adds `GAP` here, the lag or lead between the two related
    /// components, beside the RFC 5545 `RELTYPE`.
    fn allowed_params(_version: IcalVersion) -> &'static [IcalParamKind] {
        &[
            IcalParamKind::Value,
            IcalParamKind::RelType,
            IcalParamKind::Gap,
        ]
    }
}
