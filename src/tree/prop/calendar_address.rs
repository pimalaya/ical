//! # CALENDAR-ADDRESS lens
//!
//! The `CALENDAR-ADDRESS` property lens.

use crate::{
    prop::IcalPropKind,
    tree::{
        line::IcalLine,
        prop::{lens::IcalPropLens, spec::IcalPropSpec},
        value::cursor::IcalValueCursor,
    },
    value::IcalValueKind,
    value::cal_address::IcalCalAddress,
    version::IcalVersion,
};

/// The `CALENDAR-ADDRESS` property lens.
#[allow(non_camel_case_types)]
pub struct CALENDAR_ADDRESS;

impl IcalPropLens for CALENDAR_ADDRESS {
    type Target<'v> = IcalCalAddress<'v>;

    type Cursor<'c, 'a>
        = IcalValueCursor<'c, 'a>
    where
        'a: 'c;

    fn cursor<'c, 'a>(line: &'c mut IcalLine<'a>) -> IcalValueCursor<'c, 'a> {
        IcalValueCursor { line }
    }
}

impl IcalPropSpec for CALENDAR_ADDRESS {
    const KIND: IcalPropKind = IcalPropKind::CalendarAddress;

    fn allowed_versions() -> &'static [IcalVersion] {
        &[IcalVersion::V2_0]
    }

    fn allowed_values(_version: IcalVersion) -> &'static [IcalValueKind] {
        &[IcalValueKind::CalAddress]
    }
}
