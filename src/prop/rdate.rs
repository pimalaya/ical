//! # RDATE
//!
//! The `RDATE` property: the dates added to the recurrence set (RFC 5545
//! 3.8.5.2).

use crate::{
    prop::{IcalPropKind, spec::IcalPropSpec},
    value::IcalValueKind,
    version::IcalVersion,
};

/// The `RDATE` property marker.
pub struct RDATE;

impl IcalPropSpec for RDATE {
    const KIND: IcalPropKind = IcalPropKind::RDate;

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
