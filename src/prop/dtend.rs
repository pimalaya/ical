//! # DTEND
//!
//! The `DTEND` property: when the component ends (RFC 5545 3.8.2.2).

use crate::{
    prop::{IcalPropKind, cardinality::IcalPropCardinality, spec::IcalPropSpec},
    value::IcalValueKind,
    version::IcalVersion,
};

/// The `DTEND` property marker.
pub struct DTEND;

impl IcalPropSpec for DTEND {
    const KIND: IcalPropKind = IcalPropKind::DtEnd;

    fn cardinality(_version: IcalVersion) -> IcalPropCardinality {
        IcalPropCardinality::AtMostOne
    }

    fn allowed_values(_version: IcalVersion) -> &'static [IcalValueKind] {
        &[IcalValueKind::DateTime]
    }
}
