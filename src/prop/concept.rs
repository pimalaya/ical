//! # CONCEPT
//!
//! The `CONCEPT` property: a concept the component belongs to, by URI (RFC 9253
//! 6.3).

use crate::{
    prop::{IcalPropKind, spec::IcalPropSpec},
    value::IcalValueKind,
    version::IcalVersion,
};

/// The `CONCEPT` property marker.
pub struct CONCEPT;

impl IcalPropSpec for CONCEPT {
    const KIND: IcalPropKind = IcalPropKind::Concept;

    fn allowed_versions() -> &'static [IcalVersion] {
        &[IcalVersion::V2_0]
    }

    fn allowed_values(_version: IcalVersion) -> &'static [IcalValueKind] {
        &[IcalValueKind::Uri, IcalValueKind::Text]
    }
}
