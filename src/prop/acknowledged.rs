//! # ACKNOWLEDGED
//!
//! The `ACKNOWLEDGED` property: when an alarm was last dismissed, so a client
//! knows not to fire it again (RFC 9074 6.1).

use crate::{
    prop::{IcalPropKind, cardinality::IcalPropCardinality, spec::IcalPropSpec},
    value::IcalValueKind,
    version::IcalVersion,
};

/// The `ACKNOWLEDGED` property marker.
pub struct ACKNOWLEDGED;

impl IcalPropSpec for ACKNOWLEDGED {
    const KIND: IcalPropKind = IcalPropKind::Acknowledged;

    fn allowed_versions() -> &'static [IcalVersion] {
        &[IcalVersion::V2_0]
    }

    fn cardinality(_version: IcalVersion) -> IcalPropCardinality {
        IcalPropCardinality::AtMostOne
    }

    fn allowed_values(_version: IcalVersion) -> &'static [IcalValueKind] {
        &[IcalValueKind::DateTime]
    }
}
