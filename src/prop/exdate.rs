//! # EXDATE
//!
//! The `EXDATE` property: the dates removed from the recurrence set (RFC 5545
//! 3.8.5.1).

use crate::{
    prop::{IcalPropKind, spec::IcalPropSpec},
    value::IcalValueKind,
    version::IcalVersion,
};

/// The `EXDATE` property marker.
pub struct EXDATE;

impl IcalPropSpec for EXDATE {
    const KIND: IcalPropKind = IcalPropKind::ExDate;

    fn allowed_values(_version: IcalVersion) -> &'static [IcalValueKind] {
        &[IcalValueKind::DateTimeList]
    }

    /// A list whatever its items are: a declared `DATE`, `DATE-TIME` or
    /// `PERIOD` describes each item, not the value as a whole, and every item
    /// is kept as raw text.
    fn value(_version: IcalVersion, _declared: Option<IcalValueKind>) -> IcalValueKind {
        IcalValueKind::DateTimeList
    }
}
