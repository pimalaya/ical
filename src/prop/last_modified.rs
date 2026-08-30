//! # LAST-MODIFIED
//!
//! The `LAST-MODIFIED` property: when the component was last changed (RFC 5545
//! 3.8.7.3).

use crate::{
    prop::{IcalPropKind, cardinality::IcalPropCardinality, spec::IcalPropSpec},
    value::IcalValueKind,
    version::IcalVersion,
};

/// The `LAST-MODIFIED` property marker.
#[allow(non_camel_case_types)]
pub struct LAST_MODIFIED;

impl IcalPropSpec for LAST_MODIFIED {
    const KIND: IcalPropKind = IcalPropKind::LastModified;

    fn cardinality(_version: IcalVersion) -> IcalPropCardinality {
        IcalPropCardinality::AtMostOne
    }

    fn allowed_values(_version: IcalVersion) -> &'static [IcalValueKind] {
        &[IcalValueKind::DateTime]
    }
}
