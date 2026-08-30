//! # CREATED
//!
//! The `CREATED` property: when the component was first created (RFC 5545
//! 3.8.7.1).

use crate::{
    prop::{IcalPropKind, cardinality::IcalPropCardinality, spec::IcalPropSpec},
    value::IcalValueKind,
    version::IcalVersion,
};

/// The `CREATED` property marker.
pub struct CREATED;

impl IcalPropSpec for CREATED {
    const KIND: IcalPropKind = IcalPropKind::Created;

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
