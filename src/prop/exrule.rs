//! # EXRULE
//!
//! The `EXRULE` property: the rule generating dates removed from the recurrence
//! set (RFC 2445 4.8.5.2, dropped by RFC 5545 and still written in the wild).

use crate::{
    prop::{IcalPropKind, spec::IcalPropSpec},
    value::IcalValueKind,
    version::IcalVersion,
};

/// The `EXRULE` property marker.
pub struct EXRULE;

impl IcalPropSpec for EXRULE {
    const KIND: IcalPropKind = IcalPropKind::ExRule;

    fn allowed_values(_version: IcalVersion) -> &'static [IcalValueKind] {
        &[IcalValueKind::Recur]
    }
}
