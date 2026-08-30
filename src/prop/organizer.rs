//! # ORGANIZER
//!
//! The `ORGANIZER` property: who organises the component, by calendar user
//! address (RFC 5545 3.8.4.3).

use crate::{
    prop::{IcalPropKind, cardinality::IcalPropCardinality, spec::IcalPropSpec},
    value::IcalValueKind,
    version::IcalVersion,
};

/// The `ORGANIZER` property marker.
pub struct ORGANIZER;

impl IcalPropSpec for ORGANIZER {
    const KIND: IcalPropKind = IcalPropKind::Organizer;

    fn allowed_versions() -> &'static [IcalVersion] {
        &[IcalVersion::V2_0]
    }

    fn cardinality(_version: IcalVersion) -> IcalPropCardinality {
        IcalPropCardinality::AtMostOne
    }

    fn allowed_values(_version: IcalVersion) -> &'static [IcalValueKind] {
        &[IcalValueKind::CalAddress]
    }
}
